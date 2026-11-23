pub mod admin;
pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod operation;
pub mod query;
pub mod semantic;
mod transport;

pub use auth::TokenSource;
pub use config::{Config, DEFAULT_ADDR};
pub use error::{
    auth_session_invalid_reason, backup_quiesce_reason, classify_error, classify_status,
    is_auth_session_invalid, is_cluster_quiesced, is_transaction_aborted_by_quiesce,
    is_usage_limit_exceeded, is_usage_write_gate_blocked, usage_admission_reason, ClassifiedError,
    Error, ErrorKind, ErrorSeverity, Result, AUTH_ERROR_INFO_DOMAIN,
    AUTH_REASON_AUTH_SESSION_EXPIRED, AUTH_REASON_AUTH_SESSION_INVALID,
    AUTH_REASON_AUTH_SESSION_REVOKED, AUTH_REASON_REFRESH_SESSION_INVALID,
    BACKUP_ERROR_INFO_DOMAIN, BACKUP_REASON_CLUSTER_QUIESCED,
    BACKUP_REASON_TRANSACTION_ABORTED_BY_QUIESCE, USAGE_ERROR_INFO_DOMAIN,
    USAGE_REASON_LIMIT_EXCEEDED, USAGE_REASON_WRITE_GATE_BLOCKED,
};
pub use mycel as proto;

pub use admin::AdminClient;
pub use client::session::{
    is_ambiguous_commit_error, CommitTransactionOptions, IdempotentCommitBackoff,
    IdempotentCommitRetryPolicy, IdempotentCommitRetryPredicate, IdempotentTransaction,
};
pub use client::Client;
pub use operation::new_operation_id;

pub async fn dial(cfg: Config) -> Result<Client> {
    Client::dial(cfg).await
}

pub async fn dial_admin(cfg: Config) -> Result<AdminClient> {
    AdminClient::dial(cfg).await
}
