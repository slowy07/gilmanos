#[macro_use]
extern crate log;

pub mod datastore;
pub mod model;
pub mod modeled_types;
pub mod server;

pub use server::serve;