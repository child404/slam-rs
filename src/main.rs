use clap::Parser;
use slam_rs::{app, daemon, Args};
use std::process;

fn main() {
    let args = Args::parse();

    if args.daemon {
        daemon::run_daemon();
        process::exit(0);
    }

    if let Some(layout_path) = args.layout {
        app::apply_layout(&layout_path);
        process::exit(0);
    }

    let config_path = args.config.unwrap_or_else(slam_rs::find_config_path);

    app::run(&config_path, args.dmenu).unwrap_or_else(|error| match error {
        app::Error::ScreenError(error) => {
            slam_rs::exit_err!("Failed to read screen properties: {}", error)
        }
        app::Error::ConfigError(error) => slam_rs::exit_err!("{}", error),
        app::Error::CmdError(error) => slam_rs::exit_err!("{}", error),
        app::Error::InternalError => slam_rs::exit_err!("Unexpected error occured!"),
    })
}
