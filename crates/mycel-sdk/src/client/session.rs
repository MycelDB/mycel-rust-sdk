use mycel_proto::client::v1::{
    BeginTransactionRequest, CloseSessionRequest, CloseTransactionRequest,
    CommitTransactionRequest, GetDomainRequest, OpenSessionRequest, TransactionMode,
};

use crate::{
    auth::is_expired_unauthenticated,
    client::Client,
    error::{Error, Result},
};

macro_rules! client_call_with_refresh {
    ($client:ident, $call:expr, $retry:expr) => {{
        $client.refresh_if_needed().await?;
        match $call.await {
            Ok(res) => Ok(res),
            Err(status) if is_expired_unauthenticated(&status) && $client.tokens.can_refresh() => {
                $client.refresh_after_expired().await?;
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
            Err(err) if self.cfg.primary_follow.enabled && self.cfg.primary_follow.retry_reads => {
                if self.follow_primary_from_error(&err).await?.is_some() {
                    self.domain
                        .get_domain(self.auth_request(GetDomainRequest {
                            space_id,
                            domain_id: String::new(),
                            key: domain_key.clone(),
                        }))
                        .await?
                } else {
                    return Err(err);
                }
            }
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
            Err(err) if self.cfg.primary_follow.enabled && self.cfg.primary_follow.retry_reads => {
                if self.follow_primary_from_error(&err).await?.is_some() {
                    self.session
                        .open_session(self.auth_request(OpenSessionRequest {
                            space_id,
                            domain_id,
                            requested_idle_timeout: None,
                        }))
                        .await?
                } else {
                    return Err(err);
                }
            }
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
        let session_id = session_id.into();
        let res = client_call_with_refresh!(
            self,
            self.transaction
                .begin_transaction(self.auth_request(BeginTransactionRequest {
                    session_id: session_id.clone(),
                    mode: mode as i32,
                })),
            self.transaction
                .begin_transaction(self.auth_request(BeginTransactionRequest {
                    session_id,
                    mode: mode as i32,
                }))
        )?
        .into_inner();
        res.transaction.map(|tx| tx.transaction_id).ok_or_else(|| {
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

    pub async fn begin_read_only_transaction(
        &mut self,
        session_id: impl Into<String>,
    ) -> Result<String> {
        self.begin_transaction(session_id, TransactionMode::ReadOnly)
            .await
    }

    pub async fn commit_transaction(&mut self, transaction_id: impl Into<String>) -> Result<()> {
        let transaction_id = transaction_id.into();
        match client_call_with_refresh!(
            self,
            self.transaction
                .commit_transaction(self.auth_request(CommitTransactionRequest {
                    transaction_id: transaction_id.clone(),
                })),
            self.transaction
                .commit_transaction(self.auth_request(CommitTransactionRequest { transaction_id }))
        ) {
            Ok(_) => Ok(()),
            Err(err) => {
                self.follow_primary_for_unsafe(
                    "commit transaction; reopen transaction on new primary",
                    &err,
                )
                .await
            }
        }
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
