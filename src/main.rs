fn main() {
    match power_plan_sync::app_args::parse(
        std::env::args_os().skip(1),
        std::env::var_os("LOCALAPPDATA"),
    ) {
        Ok(power_plan_sync::app_args::Arguments::Help) => {
            println!("{}", power_plan_sync::app_args::HELP)
        }
        Ok(power_plan_sync::app_args::Arguments::Run(options)) => {
            #[cfg(windows)]
            if let Err(error) = power_plan_sync::runtime::run(options) {
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
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
