mod grpc;
mod model;
mod service;
mod store;

use common::telemetry::log_init;

fn main() {
    log_init("identity");
    println!("identity");
}
