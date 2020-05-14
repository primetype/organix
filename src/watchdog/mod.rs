mod control_command;
mod monitor;

pub(crate) use self::control_command::{ControlCommand, Reply};
pub use self::{control_command::WatchdogQuery, monitor::WatchdogMonitor};
use crate::{
    runtime::Runtimes,
    service::{ServiceError, ServiceIdentifier, StatusReport},
};
use async_trait::async_trait;
use std::{any::Any, fmt};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

/// trait to define the different core services and their
/// associated metadata
#[async_trait]
pub trait Organix: Send + Sync {
    fn new(_: &mut Runtimes) -> Self;

    fn stop(&mut self, service_identifier: ServiceIdentifier) -> Result<(), WatchdogError>;
    async fn status(
        &mut self,
        service_identifier: ServiceIdentifier,
    ) -> Result<StatusReport, WatchdogError>;
    fn start(
        &mut self,
        service_identifier: ServiceIdentifier,
        watchdog_query: WatchdogQuery,
    ) -> Result<(), WatchdogError>;
    fn intercoms(
        &mut self,
        service_identifier: ServiceIdentifier,
    ) -> Result<Box<dyn Any + Send + 'static>, WatchdogError>;
}

pub struct Watchdog<T: Organix> {
    services: T,
    on_drop_send: oneshot::Sender<()>,
}

pub struct WatchdogBuilder<T>
where
    T: Organix,
{
    _marker: std::marker::PhantomData<T>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WatchdogError {
    #[error("Unknown service {service_identifier}, available services are {possible_values:?}")]
    UnknownService {
        service_identifier: ServiceIdentifier,
        possible_values: &'static [ServiceIdentifier],
    },

    #[error("Cannot start service {service_identifier}: {source}")]
    CannotStartService {
        service_identifier: ServiceIdentifier,
        source: ServiceError,
    },

    #[error("Cannot connect to service {service_identifier}, service might be shutdown")]
    CannotConnectToService {
        service_identifier: ServiceIdentifier,
        retry_attempted: bool,
    },

    #[error("The watchdog didn't reply to the {context}: {reason}")]
    NoReply {
        reason: oneshot::error::RecvError,
        context: &'static str,
    },
}

impl<T> WatchdogBuilder<T>
where
    T: Organix,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    pub fn build(self) -> WatchdogMonitor
    where
        T: Organix + 'static,
    {
        let mut runtimes = Runtimes::new().unwrap();

        let services = T::new(&mut runtimes);

        let (sender, receiver) = mpsc::channel(10);
        let (on_drop_send, on_drop_receive) = oneshot::channel();

        let watchdog = Watchdog {
            on_drop_send,
            services,
        };

        let watchdog_query_handle = runtimes.watchdog().handle().clone();

        let query = WatchdogQuery::new(watchdog_query_handle, sender.clone());

        runtimes
            .watchdog()
            .handle()
            .spawn(async move { watchdog.watchdog(receiver, query).await });

        WatchdogMonitor::new(runtimes, sender, on_drop_receive)
    }
}

impl<T> Watchdog<T>
where
    T: Organix,
{
    #[tracing::instrument(skip(self, cc, watchdog_query), target = "watchdog", level = "info")]
    async fn watchdog(
        mut self,
        mut cc: mpsc::Receiver<ControlCommand>,
        watchdog_query: WatchdogQuery,
    ) {
        while let Some(command) = cc.recv().await {
            match command {
                ControlCommand::Shutdown | ControlCommand::Kill => {
                    // TODO: for now we assume shutdown and kill are the same
                    //       but on the long run it will need to send a Shutdown
                    //       signal to every services so they can save state and
                    //       release resources properly

                    tracing::warn!(%command, "stopping watchdog");
                    break;
                }
                ControlCommand::Status {
                    service_identifier,
                    reply,
                } => {
                    let status_report = self.services.status(service_identifier).await;
                    if let Ok(status_report) = &status_report {
                        tracing::info!(
                            %status_report.identifier,
                            status_report.number_restart = status_report.started,
                            %status_report.status,
                            %status_report.intercom.number_sent,
                            %status_report.intercom.number_received,
                            %status_report.intercom.number_connections,
                            %status_report.intercom.processing_speed_mean,
                            %status_report.intercom.processing_speed_variance,
                            %status_report.intercom.processing_speed_standard_derivation,
                        );
                    }
                    reply.reply(status_report);
                }
                ControlCommand::Start {
                    service_identifier,
                    reply,
                } => {
                    tracing::info!(%service_identifier, "start");
                    reply.reply(
                        self.services
                            .start(service_identifier, watchdog_query.clone()),
                    );
                }
                ControlCommand::Stop {
                    service_identifier,
                    reply,
                } => {
                    tracing::info!(%service_identifier, "stop");
                    reply.reply(self.services.stop(service_identifier));
                }
                ControlCommand::Intercom {
                    service_identifier,
                    reply,
                } => {
                    tracing::trace!(%service_identifier, "query intercom");
                    // TODO: surround the operation with a timeout and
                    //       result to success
                    reply.reply(self.services.intercoms(service_identifier));
                }
            }
        }

        if self.on_drop_send.send(()).is_err() {
            // ignore error for now
        }
    }
}

impl<T: Organix> fmt::Debug for Watchdog<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Watchdog").finish()
    }
}
