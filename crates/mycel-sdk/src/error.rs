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

    pub fn primary_hint(&self) -> Option<PrimaryHint> {
        let status = match self {
            Error::Status(status) => status,
            _ => return None,
        };
        let metadata = status.metadata();
        let value = |key: &str| {
            metadata
                .get(key)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string())
        };
        Some(PrimaryHint {
            node_id: value("mycel-primary-node-id"),
            node_name: value("mycel-primary-node-name"),
            backend_advertise_addr: value("mycel-primary-backend-advertise-addr"),
            authority_epoch: value("mycel-authority-epoch"),
        })
    }
}

pub type Result<T> = std::result::Result<T, Error>;
