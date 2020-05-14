use std::{collections::HashMap, future::Future};
use tokio::runtime::Handle;

pub struct Runtimes {
    watchdog: Runtime,
    shared: Runtime,
    individuals: HashMap<&'static str, Runtime>,
}

#[derive(Debug)]
pub struct RuntimeConfig {
    pub thread_name: &'static str,
    pub io_driver: bool,
    pub time_driver: bool,
    pub core_threads: Option<usize>,
    pub max_threads: Option<usize>,
    pub thread_stack_size: Option<usize>,
}

pub struct Runtime {
    rt: tokio::runtime::Runtime,
    config: RuntimeConfig,
}

impl Runtimes {
    pub fn new() -> std::io::Result<Self> {
        let watchdog = Runtime::build(RuntimeConfig::watchdog())?;
        let shared = Runtime::build(RuntimeConfig::shared())?;

        Ok(Self {
            watchdog,
            shared,
            individuals: HashMap::new(),
        })
    }

    pub fn watchdog(&self) -> &Runtime {
        &self.watchdog
    }

    pub fn watchdog_mut(&mut self) -> &mut Runtime {
        &mut self.watchdog
    }

    pub fn shared(&self) -> &Runtime {
        &self.shared
    }

    pub fn shared_mut(&mut self) -> &mut Runtime {
        &mut self.shared
    }

    pub fn add(&mut self, rt: Runtime) {
        self.individuals.insert(rt.config.thread_name, rt);
    }

    pub fn individual(&self, k: &'static str) -> Option<&Runtime> {
        self.individuals.get(k)
    }

    pub fn individual_mut(&mut self, k: &'static str) -> Option<&mut Runtime> {
        self.individuals.get_mut(k)
    }
}

impl Runtime {
    pub fn build(config: RuntimeConfig) -> std::io::Result<Self> {
        let mut builder = tokio::runtime::Builder::new();

        builder.thread_name(config.thread_name);

        if config.io_driver {
            builder.enable_io();
        }

        if config.time_driver {
            builder.enable_time();
        }

        if let Some(core_threads) = config.core_threads {
            builder.core_threads(core_threads);
        }

        if let Some(max_threads) = config.max_threads {
            builder.max_threads(max_threads);
        }

        if let Some(thread_stack_size) = config.thread_stack_size {
            builder.thread_stack_size(thread_stack_size);
        }

        builder
            .threaded_scheduler()
            .build()
            .map(|rt| Self { rt, config })
    }

    pub fn handle(&self) -> &Handle {
        self.rt.handle()
    }

    pub fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        self.rt.block_on(future)
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
}

impl RuntimeConfig {
    pub fn new(thread_name: &'static str) -> Self {
        Self {
            thread_name,
            io_driver: false,
            time_driver: false,
            core_threads: None,
            max_threads: None,
            thread_stack_size: None,
        }
    }

    fn watchdog() -> Self {
        Self {
            thread_name: "watchdog",
            io_driver: false,
            time_driver: true,
            core_threads: None,
            max_threads: None,
            thread_stack_size: None,
        }
    }

    fn shared() -> Self {
        Self {
            thread_name: "shared",
            io_driver: true,
            time_driver: true,
            core_threads: None,
            max_threads: None,
            thread_stack_size: None,
        }
    }
}
