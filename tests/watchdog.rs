//! this file test mainly the watchdog properties without
//! services to add noises around.
//!

use organix::{Organix, WatchdogBuilder};
use std::time::Duration;
use tokio::time::delay_for;

#[derive(Organix)]
struct NoServices;

/// test that the execution of the watchdog will be stopped shortly
/// after receiving the shutdown command from the controller
#[test]
fn start_shutdown_watchdog() {
    let watchdog = WatchdogBuilder::<NoServices>::new().build();
    let mut controller = watchdog.control();

    watchdog.spawn(async move {
        delay_for(Duration::from_millis(10)).await;
        controller.shutdown().await;
    });

    watchdog.wait_finished();
}

/// test that the execution of the watchdog will be stopped shortly
/// after receiving the kill command from the controller
#[test]
fn start_kill_watchdog() {
    let watchdog = WatchdogBuilder::<NoServices>::new().build();
    let mut controller = watchdog.control();

    watchdog.spawn(async move {
        delay_for(Duration::from_millis(10)).await;
        controller.kill().await;
    });

    watchdog.wait_finished();
}

/// starting an unknown service will fail and the error will
/// be appropriately reported back up to the monitor
#[test]
fn start_unknown_service() {
    let watchdog = WatchdogBuilder::<NoServices>::new().build();
    let mut controller = watchdog.control();

    watchdog.spawn(async move {
        delay_for(Duration::from_millis(10)).await;
        controller.kill().await;
    });

    watchdog.wait_finished()
}
