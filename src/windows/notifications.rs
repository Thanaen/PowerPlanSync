use std::fmt;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Power::{
    HPOWERNOTIFY, POWERBROADCAST_SETTING, RegisterPowerSettingNotification,
    UnregisterPowerSettingNotification,
};
use windows::Win32::System::RemoteDesktop::{
    NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification, WTSUnRegisterSessionNotification,
};
use windows::Win32::System::SystemServices::{
    GUID_ACDC_POWER_SOURCE, GUID_POWERSCHEME_PERSONALITY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CreateWindowExW, DEVICE_NOTIFY_WINDOW_HANDLE, DefWindowProcW, DestroyWindow,
    DispatchMessageW, GWLP_USERDATA, GetMessageW, GetWindowLongPtrW, MSG, PBT_POWERSETTINGCHANGE,
    PostMessageW, PostQuitMessage, RegisterClassW, SetWindowLongPtrW, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CLOSE, WM_DESTROY, WM_ENDSESSION, WM_NCCREATE, WM_POWERBROADCAST,
    WM_QUERYENDSESSION, WM_SETTINGCHANGE, WM_SYSCOLORCHANGE, WM_THEMECHANGED, WM_WTSSESSION_CHANGE,
    WNDCLASSW,
};
use windows::core::PCWSTR;

use super::power::{PowerNotification, classify_power_broadcast};
use super::session::{SessionNotification, ShutdownNotification, classify_session_change};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeNotification {
    Power(PowerNotification),
    PowerPlanChanged,
    Session(SessionNotification),
    Shutdown(ShutdownNotification),
    PreferencesChanged,
}

#[derive(Debug)]
pub struct NotificationError(String);

impl fmt::Display for NotificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for NotificationError {}

pub struct NativeNotifications {
    window: HWND,
    thread: Option<JoinHandle<()>>,
}

unsafe impl Send for NativeNotifications {}

type Callback = Box<dyn Fn(NativeNotification) + Send + 'static>;

impl NativeNotifications {
    pub fn start(
        callback: impl Fn(NativeNotification) + Send + 'static,
    ) -> Result<Self, NotificationError> {
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let thread = thread::spawn(move || run_message_loop(Box::new(callback), ready_tx));
        match ready_rx.recv() {
            Ok(Ok(window)) => Ok(Self {
                window: HWND(window as *mut _),
                thread: Some(thread),
            }),
            Ok(Err(error)) => {
                let _ = thread.join();
                Err(error)
            }
            Err(_) => {
                let _ = thread.join();
                Err(NotificationError(
                    "le thread de notifications s'est arrêté avant l'initialisation".into(),
                ))
            }
        }
    }
}

impl Drop for NativeNotifications {
    fn drop(&mut self) {
        let _ = unsafe { PostMessageW(Some(self.window), WM_CLOSE, WPARAM(0), LPARAM(0)) };
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run_message_loop(callback: Callback, ready: mpsc::SyncSender<Result<isize, NotificationError>>) {
    let callback = Box::new(callback);
    let callback_ptr = Box::into_raw(callback);
    let class_name = wide_nul(&format!(
        "PowerPlanSync.NotificationWindow.{}.{}",
        std::process::id(),
        unsafe { windows::Win32::System::Threading::GetCurrentThreadId() }
    ));
    let class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        let _ = ready.send(Err(last_error("RegisterClassW")));
        unsafe { drop(Box::from_raw(callback_ptr)) };
        return;
    }
    let window = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            None,
            None,
            None,
            Some(callback_ptr.cast()),
        )
    };
    let window = match window {
        Ok(window) => window,
        Err(error) => {
            let _ = ready.send(Err(NotificationError(format!("CreateWindowExW: {error}"))));
            unsafe { drop(Box::from_raw(callback_ptr)) };
            return;
        }
    };
    if let Err(error) = unsafe { WTSRegisterSessionNotification(window, NOTIFY_FOR_THIS_SESSION) } {
        let _ = ready.send(Err(NotificationError(format!(
            "WTSRegisterSessionNotification: {error}"
        ))));
        unsafe {
            let _ = DestroyWindow(window);
            drop(Box::from_raw(callback_ptr));
        }
        return;
    }
    let acdc = register_power(window, &GUID_ACDC_POWER_SOURCE);
    let plan = register_power(window, &GUID_POWERSCHEME_PERSONALITY);
    let (acdc, plan) = match (acdc, plan) {
        (Ok(acdc), Ok(plan)) => (acdc, plan),
        (acdc, plan) => {
            if let Ok(handle) = acdc {
                let _ = unsafe { UnregisterPowerSettingNotification(handle) };
            }
            if let Ok(handle) = plan {
                let _ = unsafe { UnregisterPowerSettingNotification(handle) };
            }
            let _ = unsafe { WTSUnRegisterSessionNotification(window) };
            let _ = ready.send(Err(NotificationError(
                "RegisterPowerSettingNotification a échoué".into(),
            )));
            unsafe {
                let _ = DestroyWindow(window);
                drop(Box::from_raw(callback_ptr));
            }
            return;
        }
    };
    if ready.send(Ok(window.0 as isize)).is_err() {
        cleanup(window, acdc, plan, callback_ptr);
        return;
    }
    let mut message = MSG::default();
    loop {
        let result = unsafe { GetMessageW(&mut message, None, 0, 0) };
        if result.0 <= 0 {
            break;
        }
        unsafe { DispatchMessageW(&message) };
    }
    cleanup(window, acdc, plan, callback_ptr);
}

fn register_power(
    window: HWND,
    setting: &windows::core::GUID,
) -> windows::core::Result<HPOWERNOTIFY> {
    unsafe {
        RegisterPowerSettingNotification(HANDLE(window.0), setting, DEVICE_NOTIFY_WINDOW_HANDLE)
    }
}

fn cleanup(window: HWND, acdc: HPOWERNOTIFY, plan: HPOWERNOTIFY, callback: *mut Callback) {
    let _ = unsafe { UnregisterPowerSettingNotification(acdc) };
    let _ = unsafe { UnregisterPowerSettingNotification(plan) };
    let _ = unsafe { WTSUnRegisterSessionNotification(window) };
    unsafe { drop(Box::from_raw(callback)) };
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, create.lpCreateParams as isize) };
    }
    let callback_ptr = unsafe { GetWindowLongPtrW(window, GWLP_USERDATA) };
    if callback_ptr != 0 {
        let callback = unsafe { &*(callback_ptr as *const Callback) };
        match message {
            WM_POWERBROADCAST if wparam.0 as u32 == PBT_POWERSETTINGCHANGE && lparam.0 != 0 => {
                let setting = unsafe { &*(lparam.0 as *const POWERBROADCAST_SETTING) };
                if setting.PowerSetting == GUID_POWERSCHEME_PERSONALITY {
                    emit(callback, NativeNotification::PowerPlanChanged);
                } else if setting.PowerSetting == GUID_ACDC_POWER_SOURCE {
                    emit(
                        callback,
                        NativeNotification::Power(PowerNotification::StatusChanged),
                    );
                }
            }
            WM_POWERBROADCAST => emit(
                callback,
                NativeNotification::Power(classify_power_broadcast(wparam.0 as u32)),
            ),
            WM_WTSSESSION_CHANGE => emit(
                callback,
                NativeNotification::Session(classify_session_change(
                    wparam.0 as u32,
                    lparam.0 as u32,
                )),
            ),
            WM_QUERYENDSESSION => emit(
                callback,
                NativeNotification::Shutdown(ShutdownNotification::Query),
            ),
            WM_ENDSESSION if wparam.0 != 0 => emit(
                callback,
                NativeNotification::Shutdown(ShutdownNotification::Ending),
            ),
            WM_SETTINGCHANGE | WM_SYSCOLORCHANGE | WM_THEMECHANGED | 0x02E0 => {
                emit(callback, NativeNotification::PreferencesChanged)
            }
            _ => {}
        }
    }
    match message {
        WM_CLOSE => {
            let _ = unsafe { DestroyWindow(window) };
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn emit(callback: &Callback, notification: NativeNotification) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(notification)));
}

fn last_error(operation: &str) -> NotificationError {
    NotificationError(format!(
        "{}: {}",
        operation,
        windows::core::Error::from_thread()
    ))
}

fn wide_nul(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_window_starts_and_stops_without_system_changes() {
        let notifications = NativeNotifications::start(|_| {}).unwrap();
        drop(notifications);
    }
}
