// Runs xrandr, parses its output,
// saves to file, and offers to apply automatically detected layout
use daemonize::Daemonize;
use std::{fs::File, thread, time};

const SAVE_DELAY: u64 = 3;

// TODO: detect monitors in live using xrandr
//
pub fn run_daemon() {
    let stdout = File::create("/tmp/slamd.out")
        .unwrap_or_else(|error| crate::exit_err!("Error creating stdout file: {}", error));
    let stderr = File::create("/tmp/slamd.err")
        .unwrap_or_else(|error| crate::exit_err!("Error creating stderr file: {}", error));
    let daemon = Daemonize::new()
        .pid_file("/tmp/slamd.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
        .umask(0o777)
        .stdout(stdout)
        .stderr(stderr);

    match daemon.start() {
        Ok(_) => loop {
            println!("Running slamd");
            thread::sleep(time::Duration::from_secs(SAVE_DELAY));
            unimplemented!();
        },
        Err(error) => {
            crate::exit_err!("Error running slamd: {}", error);
        }
    }
}
