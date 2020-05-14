use async_trait::async_trait;
use organix::{
    service, IntercomMsg, Organix, Service, ServiceIdentifier, ServiceState, WatchdogBuilder,
};
use tokio::{
    io::{stdin, stdout, AsyncBufReadExt as _, AsyncWriteExt as _, BufReader},
    stream::StreamExt as _,
};
use tracing_subscriber::fmt::Subscriber;

struct StdinReader {
    state: ServiceState<Self>,
}

struct StdoutWriter {
    state: ServiceState<Self>,
}

#[derive(Debug, IntercomMsg)]
struct WriteMsg(String);

#[async_trait]
impl Service for StdinReader {
    const SERVICE_IDENTIFIER: ServiceIdentifier = "stdin";

    type IntercomMsg = service::NoIntercom;

    fn prepare(state: ServiceState<Self>) -> Self {
        Self { state }
    }

    async fn start(mut self) {
        let mut stdout = self.state.intercom_with::<StdoutWriter>();
        let mut stdin = BufReader::new(stdin()).lines();

        while let Some(msg) = stdin.next().await {
            match msg {
                Err(err) => {
                    tracing::error!(%err);
                    break;
                }
                Ok(line) if line == "quit" => {
                    self.state.watchdog_controller().clone().shutdown().await;
                    break;
                }
                Ok(line) => {
                    tracing::debug!(%line, "read from stdin");
                    if let Err(err) = stdout.send(WriteMsg(line)).await {
                        tracing::error!(%err);
                        break;
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Service for StdoutWriter {
    const SERVICE_IDENTIFIER: ServiceIdentifier = "stdout";

    type IntercomMsg = WriteMsg;

    fn prepare(state: ServiceState<Self>) -> Self {
        Self { state }
    }

    async fn start(mut self) {
        let mut stdout = stdout();

        while let Some(WriteMsg(msg)) = self.state.intercom_mut().recv().await {
            if let Err(err) = stdout.write_all(msg.as_bytes()).await {
                tracing::error!(%err);
                break;
            }
            stdout.write_all(b"\n").await.unwrap();
            stdout.flush().await.unwrap();
        }
    }
}

#[derive(Organix)]
struct StdEcho {
    #[runtime(io)]
    stdin: service::ServiceManager<StdinReader>,
    #[runtime(io)]
    stdout: service::ServiceManager<StdoutWriter>,
}

fn main() {
    let subscriber = Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");

    let watchdog = WatchdogBuilder::<StdEcho>::new().build();

    let mut controller = watchdog.control();
    watchdog.spawn(async move {
        controller.start::<StdoutWriter>().await.unwrap();
        controller.start::<StdinReader>().await.unwrap();
    });
    watchdog.wait_finished();
}
