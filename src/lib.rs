//! # Organix, organic application
//!
//! `Organix` provides an opinionated way to build application with
//! multiple services independent from each other but still require
//! communication channels.
//!
//! With `Organix` it is possible to design the different components
//! of your application in isolation from each others. It allows to
//! build runtimes for your applications.
//!
//! # Minimal configuration
//!
//! The core component of `Organix` is the [`Watchdog`]. In order to
//! build the [`Watchdog`] you need to define an [`Organix`] object
//! which defines your app and its services.
//!
//! ```
//! use organix::{Organix, WatchdogBuilder};
//!
//! #[derive(Organix)]
//! struct App;
//!
//! let watchdog = WatchdogBuilder::<App>::new().build();
//! ```
//!
//! # defining a service
//!
//! Now defining a new service:
//!
//! ```
//! use organix::{Organix, IntercomMsg, ServiceState, Service, ServiceIdentifier, service};
//! use async_trait::async_trait;
//!
//! struct HeartBeat(ServiceState<Self>);
//!
//! #[async_trait]
//! impl Service for HeartBeat {
//!    const SERVICE_IDENTIFIER: ServiceIdentifier = "heart-beat";
//!    type IntercomMsg = service::NoIntercom;
//!
//!    fn prepare(state: ServiceState<Self>) -> Self {
//!        // initialize the state of the service
//!        Self(state)
//!    }
//!    async fn start(mut self) {
//!        // where you do the work
//!    }
//! }
//! ```
//!
//! Now from there you can start the service by adding it in the `App`:
//!
//! ```
//! use organix::{Organix, ServiceManager};
//! # use organix::{IntercomMsg, ServiceState, Service, ServiceIdentifier, service};
//! # use async_trait::async_trait;
//! #
//! # struct HeartBeat(ServiceState<Self>);
//! #
//! # #[async_trait]
//! # impl Service for HeartBeat {
//! #    const SERVICE_IDENTIFIER: ServiceIdentifier = "heart-beat";
//! #    type IntercomMsg = service::NoIntercom;
//! #
//! #    fn prepare(state: ServiceState<Self>) -> Self {
//! #        // initialize the state of the service
//! #        Self(state)
//! #    }
//! #    async fn start(mut self) {
//! #        // where you do the work
//! #    }
//! # }
//!
//! #[derive(Organix)]
//! struct App {
//!   heart_beat: service::ServiceManager<HeartBeat>,
//! }
//! ```
//!
//! See the [examples] for more complete details on how to build services
//! with the provided interface.
//!
//! # Configuring the runtime
//!
//! It is possible to configure the runtime of the different serviced.
//!
//! ## on the `Organix` app type
//!
//! * `#[runtime(shared)]`: will make all the services to use a _shared_ runtime
//!   by default. Otherwise the default is for every service to run an individual
//!   runtime.
//!
//! ## On the field of the `Organix` app type
//!
//! * `#[runtime(shared)]`: will make the associated service to use a shared runtime
//!   with the other _shared_ labeled services. This shared runtime has `io` and
//!   `time` drivers already enabled.
//! * `#[runtime(io)]`: enable the `io` driver;
//! * `#[runtime(time)]`: enable the `time` driver;
//! * `#[runtime(skip)]`: ignore the field.
//!
//! [examples]: https://github.com/primetype/organix/tree/master/examples
//! [`Watchdog`]: ./struct.WatchdogMonitor.html

pub mod runtime;
pub mod service;
mod watchdog;

pub use organix_derive::{IntercomMsg, Organix};
pub use service::{Service, ServiceIdentifier, ServiceManager, ServiceState};
pub use watchdog::{Organix, WatchdogBuilder, WatchdogError, WatchdogMonitor, WatchdogQuery};
