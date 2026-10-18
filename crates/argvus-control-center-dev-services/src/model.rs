//! Implements domain state and models consumed by the UI in crate `argvus control center dev services`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_control_center_services::Unit;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// A TCP port found listening by `ss -tln`.
pub struct ListeningPort {
  pub port: u16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// One row of `podman ps` or `docker ps`.
pub struct ContainerSummary {
  pub runtime: String,
  pub name: String,
  pub image: String,
  pub status: String,
}

#[derive(Debug, Clone, Default)]
/// The three independent sources the Dev Services page reports. Each one is
/// refreshed in its own job, so a failure in one does not hide the others.
pub struct DevServicesSnapshot {
  pub user_services: Vec<Unit>,
  pub ports: Vec<ListeningPort>,
  pub containers: Vec<ContainerSummary>,
  pub services_error: Option<String>,
  pub ports_error: Option<String>,
  pub containers_error: Option<String>,
}
