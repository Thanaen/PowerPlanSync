use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Duration;

use slint::{CloseRequestResponse, ComponentHandle, Image, Rgba8Pixel, SharedPixelBuffer};

slint::include_modules!();

type WindowSlot = Rc<RefCell<Option<ProbeWindow>>>;

fn main() -> Result<(), slint::PlatformError> {
    let Some(smoke) = parse_smoke() else {
        return Ok(());
    };
    let tray = ProbeTray::new()?;
    tray.set_tray_icon(tray_icon());
    let window: WindowSlot = Rc::new(RefCell::new(None));

    let open_slot = Rc::clone(&window);
    tray.on_open_requested(move || {
        if let Err(error) = open_window(&open_slot) {
            eprintln!("Ouverture de la fenêtre impossible: {error}");
        }
    });
    tray.on_quit_requested(|| {
        let _ = slint::quit_event_loop();
    });

    tray.show()?;
    open_window(&window)?;

    if smoke {
        let hide_slot = Rc::clone(&window);
        slint::Timer::single_shot(Duration::from_millis(400), move || {
            close_window(&hide_slot);
            let reopen_slot = Rc::clone(&hide_slot);
            slint::Timer::single_shot(Duration::from_millis(400), move || {
                if let Err(error) = open_window(&reopen_slot) {
                    eprintln!("Réouverture de la fenêtre impossible: {error}");
                }
                slint::Timer::single_shot(Duration::from_millis(400), || {
                    let _ = slint::quit_event_loop();
                });
            });
        });
    }

    slint::run_event_loop()
}

fn parse_smoke() -> Option<bool> {
    let mut smoke = false;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--smoke" => smoke = true,
            "--help" => {
                println!("ui_probe [--smoke]");
                return None;
            }
            _ => {
                eprintln!("Argument inconnu: {argument}");
                eprintln!("Usage: ui_probe [--smoke]");
                std::process::exit(2);
            }
        }
    }
    Some(smoke)
}

fn open_window(slot: &WindowSlot) -> Result<(), slint::PlatformError> {
    if let Some(window) = slot.borrow().as_ref() {
        return window.show();
    }
    let window = ProbeWindow::new()?;
    let weak_slot = Rc::downgrade(slot);
    window.on_hide_requested(move || close_weak_window(&weak_slot));
    let weak_slot = Rc::downgrade(slot);
    window.window().on_close_requested(move || {
        close_weak_window(&weak_slot);
        CloseRequestResponse::HideWindow
    });
    window.on_quit_requested(|| {
        let _ = slint::quit_event_loop();
    });
    window.show()?;
    *slot.borrow_mut() = Some(window);
    Ok(())
}

fn close_weak_window(slot: &Weak<RefCell<Option<ProbeWindow>>>) {
    if let Some(slot) = slot.upgrade() {
        close_window(&slot);
    }
}

fn close_window(slot: &WindowSlot) {
    if let Some(window) = slot.borrow_mut().take() {
        let _ = window.hide();
    }
}

fn tray_icon() -> Image {
    let mut pixels = SharedPixelBuffer::<Rgba8Pixel>::new(32, 32);
    for (index, pixel) in pixels.make_mut_slice().iter_mut().enumerate() {
        let x = index % 32;
        let y = index / 32;
        let accent = (6..26).contains(&x) && (6..26).contains(&y);
        *pixel = if accent {
            Rgba8Pixel::new(43, 105, 191, 255)
        } else {
            Rgba8Pixel::new(0, 0, 0, 0)
        };
    }
    Image::from_rgba8(pixels)
}
