use crate::{
    runtime::Runtimes,
    watchdog::{ControlCommand, WatchdogQuery},
};
use std::future::Future;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

pub struct WatchdogMonitor {
    runtimes: Runtimes,
    control_command: mpsc::Sender<ControlCommand>,
    watchdog_finished: oneshot::Receiver<()>,
}

impl WatchdogMonitor {
    pub(crate) fn new(
        runtimes: Runtimes,
        control_command: mpsc::Sender<ControlCommand>,
        watchdog_finished: oneshot::Receiver<()>,
    ) -> Self {
        WatchdogMonitor {
            runtimes,
            control_command,
            watchdog_finished,
        }
    }

    pub fn control(&self) -> WatchdogQuery {
        WatchdogQuery::new(
            self.runtimes.watchdog().handle().clone(),
            self.control_command.clone(),
        )
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtimes.watchdog().handle().spawn(future)
    }

    pub fn wait_finished(self) {
        let Self {
            mut runtimes,
            watchdog_finished,
            ..
        } = self;

        runtimes
            .watchdog_mut()
            .block_on(async move { watchdog_finished.await.unwrap() })
    }
}
