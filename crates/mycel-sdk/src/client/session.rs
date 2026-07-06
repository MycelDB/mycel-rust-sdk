use mycel_proto::client::v1::{
    BeginTransactionRequest, CloseSessionRequest, CloseTransactionRequest, CommitTransactionRequest,
    GetDomainRequest, OpenSessionRequest, TransactionMode,
};

use crate::{client::Client, error::{Error, Result}};

impl Client {
    pub async fn resolve_domain_id(&mut self, space_id: impl Into<String>, domain_key: impl Into<String>) -> Result<String> {
        let domain_key = {
            let key = domain_key.into();
            if key.is_empty() { "default".to_string() } else { key }
        };
        let res = self.domain.get_domain(self.auth_request(GetDomainRequest { space_id: space_id.into(), key: domain_key.clone() })).await?.into_inner();
        res.domain
            .map(|d| d.domain_id)
            .ok_or_else(|| Error::Message(format!("domain {domain_key:?} not found in response")))
    }

    pub async fn open_session(&mut self, space_id: impl Into<String>, domain_id: impl Into<String>) -> Result<String> {
        let res = self.session.open_session(self.auth_request(OpenSessionRequest {
            space_id: space_id.into(),
            domain_id: domain_id.into(),
            requested_idle_timeout: None,
        })).await?.into_inner();
        res.session
            .map(|s| s.session_id)
            .ok_or_else(|| Error::Message("open session response did not include a session".into()))
    }

    pub async fn close_session(&mut self, session_id: impl Into<String>) -> Result<()> {
        self.session.close_session(self.auth_request(CloseSessionRequest { session_id: session_id.into() })).await?;
        Ok(())
    }

    pub async fn begin_transaction(&mut self, session_id: impl Into<String>, mode: TransactionMode) -> Result<String> {
        let res = self.transaction.begin_transaction(self.auth_request(BeginTransactionRequest { session_id: session_id.into(), mode: mode as i32 })).await?.into_inner();
        res.transaction
            .map(|tx| tx.transaction_id)
            .ok_or_else(|| Error::Message("begin transaction response did not include a transaction".into()))
    }

    pub async fn begin_read_write_transaction(&mut self, session_id: impl Into<String>) -> Result<String> {
        self.begin_transaction(session_id, TransactionMode::ReadWrite).await
    }

    pub async fn begin_read_only_transaction(&mut self, session_id: impl Into<String>) -> Result<String> {
        self.begin_transaction(session_id, TransactionMode::ReadOnly).await
    }

    pub async fn commit_transaction(&mut self, transaction_id: impl Into<String>) -> Result<()> {
        self.transaction.commit_transaction(self.auth_request(CommitTransactionRequest { transaction_id: transaction_id.into() })).await?;
        Ok(())
    }

    pub async fn close_transaction(&mut self, transaction_id: impl Into<String>) -> Result<()> {
        self.transaction.close_transaction(self.auth_request(CloseTransactionRequest { transaction_id: transaction_id.into() })).await?;
        Ok(())
    }
}
