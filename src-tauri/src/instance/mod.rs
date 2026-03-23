#![allow(dead_code)] // Code for future phases

mod manager;
pub mod models;
mod names;
mod ports;

pub use manager::InstanceManager;
pub use models::{
    DockerState, DockerStatus, GatewayBind, InstanceConfig, InstanceSettings,
    InstanceState, InstanceStatus, InstanceWithStatus, Release,
};
pub use names::generate_name;
pub use ports::{allocate_ports, validate_port};
