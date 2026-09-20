#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() {
    match power_plan_sync::app_args::parse(
        std::env::args_os().skip(1),
        std::env::var_os("LOCALAPPDATA"),
    ) {
        Ok(power_plan_sync::app_args::Arguments::Help) => {
            attach_parent_console();
            println!("{}", power_plan_sync::app_args::HELP)
        }
        Ok(power_plan_sync::app_args::Arguments::Run(options)) => {
            #[cfg(windows)]
            if let Err(error) = power_plan_sync::runtime::run(options) {
                attach_parent_console();
                eprintln!("{error}");
                std::process::exit(1);
            }
            #[cfg(not(windows))]
            {
                let _ = options;
                eprintln!("PowerPlanSync nécessite Windows 11.");
            }
        }
        Err(error) => {
            attach_parent_console();
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

fn attach_parent_console() {
    #[cfg(windows)]
    unsafe {
        let _ = windows::Win32::System::Console::AttachConsole(
            windows::Win32::System::Console::ATTACH_PARENT_PROCESS,
        );
    }
}
