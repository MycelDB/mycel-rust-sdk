use mycel_proto::client::v1::{
    auth_service_client::AuthServiceClient, blob_service_client::BlobServiceClient,
    change_stream_service_client::ChangeStreamServiceClient,
    domain_service_client::DomainServiceClient, graph_service_client::GraphServiceClient,
    import_export_service_client::ImportExportServiceClient,
    metadata_catalog_service_client::MetadataCatalogServiceClient,
    query_service_client::QueryServiceClient, semantic_service_client::SemanticServiceClient,
    session_service_client::SessionServiceClient, space_service_client::SpaceServiceClient,
    template_service_client::TemplateServiceClient,
    transaction_service_client::TransactionServiceClient,
};

use crate::{
    error::{Error, PrimaryHint, Result},
    transport::connect_channel,
};

use super::Client;

#[derive(Debug, thiserror::Error)]
#[error("primary changed to {primary:?}; retry {operation} on new primary")]
pub struct PrimaryChangedRetryRequired {
    pub primary: PrimaryHint,
    pub operation: String,
}

impl Client {
    pub fn current_addr(&self) -> &str {
        self.cfg.addr()
    }

    pub async fn reconnect(&mut self, addr: impl Into<String>) -> Result<()> {
        self.cfg.addr = addr.into();
        let channel = connect_channel(&self.cfg).await?;
        let interceptor = self.tokens.interceptor();
        self.auth = AuthServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.space = SpaceServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.domain = DomainServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.template =
            TemplateServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.session = SessionServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.transaction =
            TransactionServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.graph = GraphServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.blob = BlobServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.query = QueryServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.import_export =
            ImportExportServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.metadata =
            MetadataCatalogServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.semantic =
            SemanticServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        self.change_stream =
            ChangeStreamServiceClient::with_interceptor(channel.clone(), interceptor);
        self.channel = channel;
        if !self.cfg.username.is_empty() || !self.cfg.password.is_empty() {
            let u = self.cfg.username.clone();
            let p = self.cfg.password.clone();
            self.login(u, p).await?;
        }
        Ok(())
    }

    pub async fn follow_primary_from_error(&mut self, err: &Error) -> Result<Option<PrimaryHint>> {
        let hint = match err.primary_hint() {
            Some(h) => h,
            None => return Ok(None),
        };
        let addr = match hint.backend_advertise_addr.clone() {
            Some(a) if !a.trim().is_empty() => a,
            _ => return Ok(None),
        };
        self.reconnect(addr).await?;
        Ok(Some(hint))
    }

    pub async fn follow_primary_for_unsafe(
        &mut self,
        operation: impl Into<String>,
        err: &Error,
    ) -> Result<()> {
        if let Some(hint) = self.follow_primary_from_error(err).await? {
            return Err(Error::Message(
                PrimaryChangedRetryRequired {
                    primary: hint,
                    operation: operation.into(),
                }
                .to_string(),
            ));
        }
        Err(Error::Message(err.to_string()))
    }
}
