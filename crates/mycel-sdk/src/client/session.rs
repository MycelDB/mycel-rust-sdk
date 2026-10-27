use std::time::Duration;

use mycel::client::v1::{
    BeginTransactionRequest, CloseSessionRequest, CloseTransactionRequest,
    CommitTransactionRequest, CommitTransactionResponse, GetDomainRequest, GraphTransaction,
    OpenSessionRequest, TransactionCommit, TransactionMode,
};

use crate::{
    auth::is_expired_unauthenticated,
    client::Client,
    error::{Error, Result},
};

/// Durable transaction-level idempotency scope. Callers must preserve and
/// reuse the exact values after an ambiguous commit outcome.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitTransactionOptions {
    pub idempotency_key: String,
    pub space_id: String,
    pub domain_id: String,
}

/// A read-write transaction paired with the durable commit scope selected
/// before graph mutations are staged.
#[derive(Clone, Debug, PartialEq)]
pub struct IdempotentTransaction {
    pub transaction: GraphTransaction,
    pub commit_options: CommitTransactionOptions,
}

pub type IdempotentCommitRetryPredicate = fn(&Error) -> bool;
pub type IdempotentCommitBackoff = fn(usize) -> Duration;

/// Controls opt-in retries of ambiguous idempotent commit outcomes.
/// `max_attempts` includes the first call; zero performs one attempt.
#[derive(Clone, Copy, Debug, Default)]
pub struct IdempotentCommitRetryPolicy {
    pub max_attempts: usize,
    pub retryable: Option<IdempotentCommitRetryPredicate>,
    pub backoff: Option<IdempotentCommitBackoff>,
}

/// Reports failures for which the daemon may have durably committed without
/// the client receiving the result. Explicit conflicts, capacity failures,
/// and validation failures are not ambiguous.
pub fn is_ambiguous_commit_error(err: &Error) -> bool {
    match err {
        Error::Transport(_) => true,
        Error::Status(status) => matches!(
            status.code(),
            tonic::Code::Unavailable
                | tonic::Code::DeadlineExceeded
                | tonic::Code::Unknown
                | tonic::Code::Internal
        ),
        _ => false,
    }
}

macro_rules! client_call_with_refresh {
    ($client:ident, $call:expr, $retry:expr) => {{
        $client.refresh_if_needed().await?;
        let generation = $client.tokens.generation();
        match $call.await {
            Ok(res) => Ok(res),
            Err(status) if is_expired_unauthenticated(&status) && $client.tokens.can_refresh() => {
                $client.refresh_after_expired(generation).await?;
                Ok($retry.await?)
            }
            Err(status) => Err(Error::from(status)),
        }
    }};
}

impl Client {
    pub async fn resolve_domain_id(
        &mut self,
        space_id: impl Into<String>,
        domain_key: impl Into<String>,
    ) -> Result<String> {
        let space_id = space_id.into();
        let domain_key = {
            let key = domain_key.into();
            if key.is_empty() {
                "default".to_string()
            } else {
                key
            }
        };
        let first_space_id = space_id.clone();
        let first_domain_key = domain_key.clone();
        let res = match client_call_with_refresh!(
            self,
            self.domain.get_domain(self.auth_request(GetDomainRequest {
                space_id: first_space_id,
                domain_id: String::new(),
                key: first_domain_key,
            })),
            self.domain.get_domain(self.auth_request(GetDomainRequest {
                space_id: space_id.clone(),
                domain_id: String::new(),
                key: domain_key.clone(),
            }))
        ) {
            Ok(res) => res,
            Err(err) => return Err(err),
        }
        .into_inner();
        res.domain
            .map(|d| d.domain_id)
            .ok_or_else(|| Error::Message(format!("domain {domain_key:?} not found in response")))
    }

    pub async fn open_session(
        &mut self,
        space_id: impl Into<String>,
        domain_id: impl Into<String>,
    ) -> Result<String> {
        let space_id = space_id.into();
        let domain_id = domain_id.into();
        let res = match client_call_with_refresh!(
            self,
            self.session
                .open_session(self.auth_request(OpenSessionRequest {
                    space_id: space_id.clone(),
                    domain_id: domain_id.clone(),
                    requested_idle_timeout: None,
                })),
            self.session
                .open_session(self.auth_request(OpenSessionRequest {
                    space_id: space_id.clone(),
                    domain_id: domain_id.clone(),
                    requested_idle_timeout: None,
                }))
        ) {
            Ok(res) => res,
            Err(err) => return Err(err),
        }
        .into_inner();
        res.session
            .map(|s| s.session_id)
            .ok_or_else(|| Error::Message("open session response did not include a session".into()))
    }

    pub async fn close_session(&mut self, session_id: impl Into<String>) -> Result<()> {
        let session_id = session_id.into();
        client_call_with_refresh!(
            self,
            self.session
                .close_session(self.auth_request(CloseSessionRequest {
                    session_id: session_id.clone(),
                })),
            self.session
                .close_session(self.auth_request(CloseSessionRequest { session_id }))
        )?;
        Ok(())
    }

    pub async fn begin_transaction(
        &mut self,
        session_id: impl Into<String>,
        mode: TransactionMode,
    ) -> Result<String> {
        let tx = self
            .begin_transaction_with_operation_id(session_id, mode, "")
            .await?;
        Ok(tx.transaction_id)
    }

    /// Begins a transaction with optional client operation correlation metadata.
    /// Pass [`crate::new_operation_id`] to correlate the write with later graph-change
    /// events; pass an empty string to let the daemon generate an operation ID. The
    /// returned transaction includes the resolved operation ID.
    pub async fn begin_transaction_with_operation_id(
        &mut self,
        session_id: impl Into<String>,
        mode: TransactionMode,
        operation_id: impl Into<String>,
    ) -> Result<GraphTransaction> {
        let session_id = session_id.into();
        let operation_id = operation_id.into();
        let res = client_call_with_refresh!(
            self,
            self.transaction
                .begin_transaction(self.auth_request(BeginTransactionRequest {
                    session_id: session_id.clone(),
                    mode: mode as i32,
                    operation_id: operation_id.clone(),
                })),
            self.transaction
                .begin_transaction(self.auth_request(BeginTransactionRequest {
                    session_id,
                    mode: mode as i32,
                    operation_id,
                }))
        )?
        .into_inner();
        res.transaction.ok_or_else(|| {
            Error::Message("begin transaction response did not include a transaction".into())
        })
    }

    pub async fn begin_read_write_transaction(
        &mut self,
        session_id: impl Into<String>,
    ) -> Result<String> {
        self.begin_transaction(session_id, TransactionMode::ReadWrite)
            .await
    }

    pub async fn begin_read_write_transaction_with_operation_id(
        &mut self,
        session_id: impl Into<String>,
        operation_id: impl Into<String>,
    ) -> Result<GraphTransaction> {
        self.begin_transaction_with_operation_id(
            session_id,
            TransactionMode::ReadWrite,
            operation_id,
        )
        .await
    }

    pub async fn begin_read_only_transaction(
        &mut self,
        session_id: impl Into<String>,
    ) -> Result<String> {
        self.begin_transaction(session_id, TransactionMode::ReadOnly)
            .await
    }

    pub async fn begin_read_only_transaction_with_operation_id(
        &mut self,
        session_id: impl Into<String>,
        operation_id: impl Into<String>,
    ) -> Result<GraphTransaction> {
        self.begin_transaction_with_operation_id(
            session_id,
            TransactionMode::ReadOnly,
            operation_id,
        )
        .await
    }

    /// Begins an idempotent read-write transaction while retaining the scope
    /// that must be reused for an ambiguous commit outcome.
    pub async fn begin_idempotent_read_write_transaction(
        &mut self,
        session_id: impl Into<String>,
        space_id: impl Into<String>,
        domain_id: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> Result<IdempotentTransaction> {
        self.begin_idempotent_read_write_transaction_with_operation_id(
            session_id,
            space_id,
            domain_id,
            idempotency_key,
            "",
        )
        .await
    }

    pub async fn begin_idempotent_read_write_transaction_with_operation_id(
        &mut self,
        session_id: impl Into<String>,
        space_id: impl Into<String>,
        domain_id: impl Into<String>,
        idempotency_key: impl Into<String>,
        operation_id: impl Into<String>,
    ) -> Result<IdempotentTransaction> {
        let commit_options = CommitTransactionOptions {
            idempotency_key: idempotency_key.into(),
            space_id: space_id.into(),
            domain_id: domain_id.into(),
        };
        validate_commit_transaction_options(&commit_options)?;
        let transaction = self
            .begin_read_write_transaction_with_operation_id(session_id, operation_id)
            .await?;
        Ok(IdempotentTransaction {
            transaction,
            commit_options,
        })
    }

    pub async fn commit_transaction(&mut self, transaction_id: impl Into<String>) -> Result<()> {
        self.commit_transaction_result(transaction_id)
            .await
            .map(|_| ())
    }

    /// Commits a read-write transaction and returns commit metadata, including the
    /// operation ID associated with the transaction.
    pub async fn commit_transaction_result(
        &mut self,
        transaction_id: impl Into<String>,
    ) -> Result<TransactionCommit> {
        let res = self
            .commit_transaction_with_options(transaction_id, CommitTransactionOptions::default())
            .await?;
        res.commit.ok_or_else(|| {
            Error::Message("commit transaction response did not include a commit".into())
        })
    }

    /// Commits using a caller-provided durable idempotency key and scope.
    pub async fn commit_transaction_idempotent(
        &mut self,
        transaction_id: impl Into<String>,
        space_id: impl Into<String>,
        domain_id: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> Result<CommitTransactionResponse> {
        self.commit_transaction_with_options(
            transaction_id,
            CommitTransactionOptions {
                idempotency_key: idempotency_key.into(),
                space_id: space_id.into(),
                domain_id: domain_id.into(),
            },
        )
        .await
    }

    /// Commits with explicit options. The SDK never invents or rotates keys.
    pub async fn commit_transaction_with_options(
        &mut self,
        transaction_id: impl Into<String>,
        options: CommitTransactionOptions,
    ) -> Result<CommitTransactionResponse> {
        validate_commit_transaction_options(&options)?;
        let request = commit_transaction_request(transaction_id.into(), options);
        let res = client_call_with_refresh!(
            self,
            self.transaction
                .commit_transaction(self.auth_request(request.clone())),
            self.transaction
                .commit_transaction(self.auth_request(request))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn commit_idempotent_transaction(
        &mut self,
        transaction: &IdempotentTransaction,
    ) -> Result<CommitTransactionResponse> {
        if transaction.transaction.transaction_id.is_empty() {
            return Err(Error::Message("idempotent transaction is required".into()));
        }
        self.commit_transaction_with_options(
            transaction.transaction.transaction_id.clone(),
            transaction.commit_options.clone(),
        )
        .await
    }

    /// Retries only caller-approved ambiguous failures while preserving the
    /// exact transaction ID, key, space, and domain on every attempt.
    pub async fn commit_idempotent_transaction_with_retry(
        &mut self,
        transaction: &IdempotentTransaction,
        policy: IdempotentCommitRetryPolicy,
    ) -> Result<CommitTransactionResponse> {
        if transaction.transaction.transaction_id.is_empty() {
            return Err(Error::Message("idempotent transaction is required".into()));
        }
        let max_attempts = policy.max_attempts.max(1);
        let retryable = policy.retryable.unwrap_or(is_ambiguous_commit_error);
        for attempt in 1..=max_attempts {
            match self.commit_idempotent_transaction(transaction).await {
                Ok(response) => return Ok(response),
                Err(err) if attempt < max_attempts && retryable(&err) => {
                    if let Some(backoff) = policy.backoff {
                        let delay = backoff(attempt);
                        if !delay.is_zero() {
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
                Err(err) => return Err(err),
            }
        }
        unreachable!("max_attempts is at least one")
    }

    pub async fn close_transaction(&mut self, transaction_id: impl Into<String>) -> Result<()> {
        let transaction_id = transaction_id.into();
        client_call_with_refresh!(
            self,
            self.transaction
                .close_transaction(self.auth_request(CloseTransactionRequest {
                    transaction_id: transaction_id.clone(),
                })),
            self.transaction
                .close_transaction(self.auth_request(CloseTransactionRequest { transaction_id }))
        )?;
        Ok(())
    }
}

fn validate_commit_transaction_options(options: &CommitTransactionOptions) -> Result<()> {
    if options.idempotency_key.is_empty() {
        if !options.space_id.is_empty() || !options.domain_id.is_empty() {
            return Err(Error::Message(
                "space ID and domain ID require an idempotency key".into(),
            ));
        }
        return Ok(());
    }
    if options.idempotency_key.trim() != options.idempotency_key
        || options.idempotency_key.len() > 128
    {
        return Err(Error::Message(
            "idempotency key must not have surrounding whitespace and must be at most 128 bytes"
                .into(),
        ));
    }
    if options.space_id.trim().is_empty() || options.domain_id.trim().is_empty() {
        return Err(Error::Message(
            "space ID and domain ID are required with an idempotency key".into(),
        ));
    }
    Ok(())
}

fn commit_transaction_request(
    transaction_id: String,
    options: CommitTransactionOptions,
) -> CommitTransactionRequest {
    CommitTransactionRequest {
        transaction_id,
        idempotency_key: options.idempotency_key,
        space_id: options.space_id,
        domain_id: options.domain_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotent_commit_request_preserves_transaction_key_and_scope() {
        let request = commit_transaction_request(
            "tx-1".into(),
            CommitTransactionOptions {
                idempotency_key: "capture-1".into(),
                space_id: "space-1".into(),
                domain_id: "domain-1".into(),
            },
        );
        assert_eq!(request.transaction_id, "tx-1");
        assert_eq!(request.idempotency_key, "capture-1");
        assert_eq!(request.space_id, "space-1");
        assert_eq!(request.domain_id, "domain-1");
    }

    #[test]
    fn idempotent_commit_options_validate_scope_and_key() {
        assert!(validate_commit_transaction_options(&CommitTransactionOptions::default()).is_ok());
        assert!(
            validate_commit_transaction_options(&CommitTransactionOptions {
                idempotency_key: "capture-1".into(),
                space_id: "space-1".into(),
                domain_id: "domain-1".into(),
            })
            .is_ok()
        );
        assert!(
            validate_commit_transaction_options(&CommitTransactionOptions {
                idempotency_key: " capture-1".into(),
                space_id: "space-1".into(),
                domain_id: "domain-1".into(),
            })
            .is_err()
        );
        assert!(
            validate_commit_transaction_options(&CommitTransactionOptions {
                idempotency_key: "capture-1".into(),
                space_id: String::new(),
                domain_id: "domain-1".into(),
            })
            .is_err()
        );
    }

    #[test]
    fn ambiguous_commit_errors_exclude_explicit_conflicts() {
        assert!(is_ambiguous_commit_error(&Error::from(tonic::Status::new(
            tonic::Code::Unavailable,
            "lost response",
        ))));
        assert!(is_ambiguous_commit_error(&Error::from(tonic::Status::new(
            tonic::Code::Internal,
            "unknown outcome",
        ))));
        assert!(!is_ambiguous_commit_error(&Error::from(
            tonic::Status::new(tonic::Code::AlreadyExists, "key mismatch",)
        )));
        assert!(!is_ambiguous_commit_error(&Error::from(
            tonic::Status::new(tonic::Code::ResourceExhausted, "capacity",)
        )));
    }
}
