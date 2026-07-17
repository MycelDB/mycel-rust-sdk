use std::collections::HashMap;

use prost::Message;
use prost_types::Any;

#[derive(Clone, PartialEq, Message)]
struct RpcStatus {
    #[prost(int32, tag = "1")]
    code: i32,
    #[prost(string, tag = "2")]
    message: String,
    #[prost(message, repeated, tag = "3")]
    details: Vec<Any>,
}

#[derive(Clone, PartialEq, Message)]
struct ErrorInfo {
    #[prost(string, tag = "1")]
    reason: String,
    #[prost(string, tag = "2")]
    domain: String,
    #[prost(map = "string, string", tag = "3")]
    metadata: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid endpoint {0:?}")]
    InvalidEndpoint(String),

    #[error("TLS insecure skip verify is not supported by the Rust SDK yet")]
    InsecureSkipVerifyUnsupported,

    #[error("TLS client cert and key must be set together")]
    PartialClientCertificate,

    #[error("read {path}: {source}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("gRPC status: {0}")]
    Status(Box<tonic::Status>),

    #[error("{0}")]
    Message(String),
}

impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Self {
        Self::Status(Box::new(status))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrimaryHint {
    pub node_id: Option<String>,
    pub node_name: Option<String>,
    pub backend_advertise_addr: Option<String>,
    pub authority_epoch: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SnapshotRequiredInfo {
    pub requested_after_lsn: Option<String>,
    pub next_requested_lsn: Option<String>,
    pub first_retained_lsn: Option<String>,
    pub last_committed_lsn: Option<String>,
    pub checkpoint_lsn: Option<String>,
    pub primary_node_id: Option<String>,
    pub authority_epoch: Option<String>,
}

impl Error {
    pub fn is_not_primary(&self) -> bool {
        match self {
            Error::Status(status) => {
                status.code() == tonic::Code::FailedPrecondition
                    && status.message() == "node is not cluster primary"
            }
            _ => false,
        }
    }

    pub fn is_snapshot_required(&self) -> bool {
        match self {
            Error::Status(status) => {
                status.code() == tonic::Code::FailedPrecondition
                    && status.message() == "follower requires snapshot catch-up"
            }
            _ => false,
        }
    }

    pub fn snapshot_required_info(&self) -> Option<SnapshotRequiredInfo> {
        let status = match self {
            Error::Status(status) => status,
            _ => return None,
        };
        self.error_info_metadata("MYCEL_WAL_SNAPSHOT_REQUIRED")
            .map(|metadata| SnapshotRequiredInfo {
                requested_after_lsn: metadata.get("requested_after_lsn").cloned(),
                next_requested_lsn: metadata.get("next_requested_lsn").cloned(),
                first_retained_lsn: metadata.get("first_retained_lsn").cloned(),
                last_committed_lsn: metadata.get("last_committed_lsn").cloned(),
                checkpoint_lsn: metadata.get("checkpoint_lsn").cloned(),
                primary_node_id: metadata.get("primary_node_id").cloned(),
                authority_epoch: metadata.get("authority_epoch").cloned(),
            })
            .or_else(|| {
                let _ = status;
                None
            })
    }

    pub fn primary_hint(&self) -> Option<PrimaryHint> {
        let status = match self {
            Error::Status(status) => status,
            _ => return None,
        };
        if let Some(metadata) = self.error_info_metadata("MYCEL_CLUSTER_NOT_PRIMARY") {
            return Some(PrimaryHint {
                node_id: metadata.get("mycel-primary-node-id").cloned(),
                node_name: metadata.get("mycel-primary-node-name").cloned(),
                backend_advertise_addr: metadata
                    .get("mycel-primary-backend-advertise-addr")
                    .cloned(),
                authority_epoch: metadata.get("mycel-authority-epoch").cloned(),
            });
        }
        let metadata = status.metadata();
        let value = |key: &str| {
            metadata
                .get(key)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string())
        };
        let hint = PrimaryHint {
            node_id: value("mycel-primary-node-id"),
            node_name: value("mycel-primary-node-name"),
            backend_advertise_addr: value("mycel-primary-backend-advertise-addr"),
            authority_epoch: value("mycel-authority-epoch"),
        };
        if hint == PrimaryHint::default() {
            None
        } else {
            Some(hint)
        }
    }

    fn error_info_metadata(&self, reason: &str) -> Option<HashMap<String, String>> {
        let status = match self {
            Error::Status(status) => status,
            _ => return None,
        };
        let rpc_status = RpcStatus::decode(status.details()).ok()?;
        for detail in rpc_status.details {
            if detail.type_url != "type.googleapis.com/google.rpc.ErrorInfo" {
                continue;
            }
            if let Ok(info) = ErrorInfo::decode(detail.value.as_slice()) {
                if info.reason == reason {
                    return Some(info.metadata);
                }
            }
        }
        None
    }
}

pub type Result<T> = std::result::Result<T, Error>;
