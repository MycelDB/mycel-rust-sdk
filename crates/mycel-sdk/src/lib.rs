pub mod admin;
pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod operation;
pub mod query;
mod transport;

pub use auth::TokenSource;
pub use config::{Config, DEFAULT_ADDR};
pub use error::{Error, Result};
pub use mycel_proto as proto;

pub use admin::AdminClient;
pub use client::Client;
pub use operation::new_operation_id;

pub async fn dial(cfg: Config) -> Result<Client> {
    Client::dial(cfg).await
}

pub async fn dial_admin(cfg: Config) -> Result<AdminClient> {
    AdminClient::dial(cfg).await
}
