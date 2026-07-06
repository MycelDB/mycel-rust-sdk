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

pub type Result<T> = std::result::Result<T, Error>;
