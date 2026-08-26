use std::collections::HashMap;
use std::error::Error as StdError;
use std::io;

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

/// Stable error category for SDK and downstream application errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Validation,
    Connectivity,
    Authentication,
    Authorization,
    NotFound,
    Conflict,
    RateLimited,
    Unavailable,
    Timeout,
    Internal,
    Unknown,
}

impl ErrorKind {
    /// Returns the stable snake_case representation for this error kind.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorKind::Validation => "validation",
            ErrorKind::Connectivity => "connectivity",
            ErrorKind::Authentication => "authentication",
            ErrorKind::Authorization => "authorization",
            ErrorKind::NotFound => "not_found",
            ErrorKind::Conflict => "conflict",
            ErrorKind::RateLimited => "rate_limited",
            ErrorKind::Unavailable => "unavailable",
            ErrorKind::Timeout => "timeout",
            ErrorKind::Internal => "internal",
            ErrorKind::Unknown => "unknown",
        }
    }

    /// Returns the default presentation severity for this error kind.
    pub fn default_severity(self) -> ErrorSeverity {
        match self {
            ErrorKind::Validation
            | ErrorKind::Authentication
            | ErrorKind::Authorization
            | ErrorKind::NotFound
            | ErrorKind::Conflict
            | ErrorKind::RateLimited => ErrorSeverity::Warning,
            ErrorKind::Connectivity
            | ErrorKind::Unavailable
            | ErrorKind::Timeout
            | ErrorKind::Internal
            | ErrorKind::Unknown => ErrorSeverity::Error,
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Presentation severity hint for a classified SDK error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
}

impl ErrorSeverity {
    /// Returns the stable snake_case representation for this severity.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorSeverity::Info => "info",
            ErrorSeverity::Warning => "warning",
            ErrorSeverity::Error => "error",
        }
    }
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// SDK error classification with stable kind, severity, and preserved detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedError {
    pub kind: ErrorKind,
    pub severity: ErrorSeverity,
    pub message: String,
    pub detail: Option<String>,
}

impl ClassifiedError {
    fn new(kind: ErrorKind, message: impl Into<String>, detail: Option<String>) -> Self {
        let message = message.into();
        Self {
            kind,
            severity: kind.default_severity(),
            message,
            detail,
        }
    }
}

/// Classifies a gRPC status into a stable SDK error category.
pub fn classify_status(status: &tonic::Status) -> ClassifiedError {
    let kind = match status.code() {
        tonic::Code::InvalidArgument => ErrorKind::Validation,
        tonic::Code::Unauthenticated => ErrorKind::Authentication,
        tonic::Code::PermissionDenied => ErrorKind::Authorization,
        tonic::Code::NotFound => ErrorKind::NotFound,
        tonic::Code::AlreadyExists | tonic::Code::Aborted => ErrorKind::Conflict,
        tonic::Code::ResourceExhausted => ErrorKind::RateLimited,
        tonic::Code::Unavailable => ErrorKind::Unavailable,
        tonic::Code::DeadlineExceeded => ErrorKind::Timeout,
        tonic::Code::Internal | tonic::Code::DataLoss => ErrorKind::Internal,
        _ => ErrorKind::Unknown,
    };
    let message = if status.message().trim().is_empty() {
        status.code().to_string()
    } else {
        status.message().to_string()
    };
    let detail = status.to_string();
    ClassifiedError::new(kind, message.clone(), (detail != message).then_some(detail))
}

/// Classifies an SDK or lower-level error into a stable category.
pub fn classify_error(err: &(dyn StdError + 'static)) -> ClassifiedError {
    if let Some(classified) = classify_direct_error(err) {
        return classified;
    }
    let mut source = err.source();
    while let Some(current) = source {
        if let Some(classified) = classify_direct_error(current) {
            return classified;
        }
        source = current.source();
    }
    ClassifiedError::new(
        ErrorKind::Unknown,
        non_empty_message(err.to_string(), "unknown error"),
        None,
    )
}

fn classify_direct_error(err: &(dyn StdError + 'static)) -> Option<ClassifiedError> {
    if let Some(sdk) = err.downcast_ref::<Error>() {
        return Some(classify_sdk_error(sdk));
    }
    if let Some(status) = err.downcast_ref::<tonic::Status>() {
        return Some(classify_status(status));
    }
    if err.downcast_ref::<tonic::transport::Error>().is_some() {
        return Some(ClassifiedError::new(
            ErrorKind::Connectivity,
            "transport error",
            Some(err.to_string()),
        ));
    }
    if let Some(io) = err.downcast_ref::<io::Error>() {
        return Some(classify_io_error(io));
    }
    if err.downcast_ref::<tokio::time::error::Elapsed>().is_some() {
        return Some(ClassifiedError::new(
            ErrorKind::Timeout,
            "operation timed out",
            Some(err.to_string()),
        ));
    }
    None
}

fn classify_sdk_error(err: &Error) -> ClassifiedError {
    match err {
        Error::InvalidEndpoint(_)
        | Error::InsecureSkipVerifyUnsupported
        | Error::PartialClientCertificate
        | Error::ReadFile { .. } => {
            ClassifiedError::new(ErrorKind::Validation, err.to_string(), None)
        }
        Error::Transport(_) => ClassifiedError::new(
            ErrorKind::Connectivity,
            "transport error",
            Some(err.to_string()),
        ),
        Error::Status(status) => classify_status(status),
        Error::Message(message) => ClassifiedError::new(
            ErrorKind::Unknown,
            non_empty_message(message, "unknown error"),
            None,
        ),
    }
}

fn classify_io_error(err: &io::Error) -> ClassifiedError {
    let kind = match err.kind() {
        io::ErrorKind::TimedOut => ErrorKind::Timeout,
        io::ErrorKind::ConnectionRefused
        | io::ErrorKind::ConnectionReset
        | io::ErrorKind::ConnectionAborted
        | io::ErrorKind::NotConnected
        | io::ErrorKind::AddrInUse
        | io::ErrorKind::AddrNotAvailable
        | io::ErrorKind::BrokenPipe
        | io::ErrorKind::UnexpectedEof => ErrorKind::Connectivity,
        _ => ErrorKind::Unknown,
    };
    ClassifiedError::new(kind, err.to_string(), None)
}

fn non_empty_message(message: impl AsRef<str>, fallback: &str) -> String {
    let message = message.as_ref().trim();
    if message.is_empty() {
        fallback.to_string()
    } else {
        message.to_string()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[derive(Debug, thiserror::Error)]
    #[error("outer: {source}")]
    struct WrappedSdkError {
        #[source]
        source: Error,
    }

    #[test]
    fn classify_status_maps_grpc_codes() {
        let cases = [
            (
                tonic::Code::InvalidArgument,
                ErrorKind::Validation,
                ErrorSeverity::Warning,
            ),
            (
                tonic::Code::Unauthenticated,
                ErrorKind::Authentication,
                ErrorSeverity::Warning,
            ),
            (
                tonic::Code::PermissionDenied,
                ErrorKind::Authorization,
                ErrorSeverity::Warning,
            ),
            (
                tonic::Code::NotFound,
                ErrorKind::NotFound,
                ErrorSeverity::Warning,
            ),
            (
                tonic::Code::AlreadyExists,
                ErrorKind::Conflict,
                ErrorSeverity::Warning,
            ),
            (
                tonic::Code::Aborted,
                ErrorKind::Conflict,
                ErrorSeverity::Warning,
            ),
            (
                tonic::Code::ResourceExhausted,
                ErrorKind::RateLimited,
                ErrorSeverity::Warning,
            ),
            (
                tonic::Code::Unavailable,
                ErrorKind::Unavailable,
                ErrorSeverity::Error,
            ),
            (
                tonic::Code::DeadlineExceeded,
                ErrorKind::Timeout,
                ErrorSeverity::Error,
            ),
            (
                tonic::Code::Internal,
                ErrorKind::Internal,
                ErrorSeverity::Error,
            ),
            (
                tonic::Code::DataLoss,
                ErrorKind::Internal,
                ErrorSeverity::Error,
            ),
            (
                tonic::Code::Cancelled,
                ErrorKind::Unknown,
                ErrorSeverity::Error,
            ),
        ];
        for (code, want_kind, want_severity) in cases {
            let classified = classify_status(&tonic::Status::new(code, "classified message"));
            assert_eq!(classified.kind, want_kind, "{code:?}");
            assert_eq!(classified.severity, want_severity, "{code:?}");
            assert_eq!(classified.message, "classified message");
        }
    }

    #[test]
    fn kind_and_severity_have_stable_strings() {
        assert_eq!(ErrorKind::Connectivity.as_str(), "connectivity");
        assert_eq!(ErrorKind::Authentication.to_string(), "authentication");
        assert_eq!(ErrorSeverity::Warning.as_str(), "warning");
        assert_eq!(ErrorSeverity::Error.to_string(), "error");
    }

    #[test]
    fn classify_sdk_status_error() {
        let err = Error::from(tonic::Status::unauthenticated("invalid credentials"));
        let classified = classify_error(&err);
        assert_eq!(classified.kind, ErrorKind::Authentication);
        assert_eq!(classified.severity, ErrorSeverity::Warning);
        assert_eq!(classified.message, "invalid credentials");
    }

    #[test]
    fn classify_wrapped_sdk_error() {
        let err = WrappedSdkError {
            source: Error::from(tonic::Status::permission_denied("denied")),
        };
        let classified = classify_error(&err);
        assert_eq!(classified.kind, ErrorKind::Authorization);
        assert_eq!(classified.severity, ErrorSeverity::Warning);
    }

    #[test]
    fn classify_io_connectivity_and_timeout() {
        let refused = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
        let classified = classify_error(&refused);
        assert_eq!(classified.kind, ErrorKind::Connectivity);
        assert_eq!(classified.severity, ErrorSeverity::Error);

        let timed_out = io::Error::new(io::ErrorKind::TimedOut, "timed out");
        let classified = classify_error(&timed_out);
        assert_eq!(classified.kind, ErrorKind::Timeout);
        assert_eq!(classified.severity, ErrorSeverity::Error);
    }

    #[test]
    fn classify_local_sdk_validation_and_unknown_message() {
        let invalid_endpoint = Error::InvalidEndpoint("%%%".to_string());
        let classified = classify_error(&invalid_endpoint);
        assert_eq!(classified.kind, ErrorKind::Validation);
        assert_eq!(classified.severity, ErrorSeverity::Warning);

        let message = Error::Message("opaque".to_string());
        let classified = classify_error(&message);
        assert_eq!(classified.kind, ErrorKind::Unknown);
        assert_eq!(classified.severity, ErrorSeverity::Error);
    }
}
