use mycel_proto::client::v1::{
    auth_service_client::AuthServiceClient, blob_service_client::BlobServiceClient,
    change_stream_service_client::ChangeStreamServiceClient, domain_service_client::DomainServiceClient,
    graph_service_client::GraphServiceClient, import_export_service_client::ImportExportServiceClient,
    metadata_catalog_service_client::MetadataCatalogServiceClient, query_service_client::QueryServiceClient,
    semantic_service_client::SemanticServiceClient, session_service_client::SessionServiceClient,
    space_service_client::SpaceServiceClient, template_service_client::TemplateServiceClient,
    transaction_service_client::TransactionServiceClient, AuthPrincipal, ClientInfo, LoginRequest,
    LoginResponse, RefreshRequest, RefreshResponse, WhoAmIRequest,
};
use tonic::{service::interceptor::InterceptedService, transport::Channel, Request};

use crate::{auth::{AuthInterceptor, TokenSource}, config::Config, error::{Error, Result}, transport::connect_channel};

pub type AuthenticatedService = InterceptedService<Channel, AuthInterceptor>;

#[derive(Debug, Clone)]
pub struct PrincipalInfo {
    pub user_id: String,
    pub username: String,
}

#[derive(Clone)]
pub struct Client {
    pub auth: AuthServiceClient<AuthenticatedService>,
    pub space: SpaceServiceClient<AuthenticatedService>,
    pub domain: DomainServiceClient<AuthenticatedService>,
    pub template: TemplateServiceClient<AuthenticatedService>,
    pub session: SessionServiceClient<AuthenticatedService>,
    pub transaction: TransactionServiceClient<AuthenticatedService>,
    pub graph: GraphServiceClient<AuthenticatedService>,
    pub blob: BlobServiceClient<AuthenticatedService>,
    pub query: QueryServiceClient<AuthenticatedService>,
    pub import_export: ImportExportServiceClient<AuthenticatedService>,
    pub metadata: MetadataCatalogServiceClient<AuthenticatedService>,
    pub semantic: SemanticServiceClient<AuthenticatedService>,
    pub change_stream: ChangeStreamServiceClient<AuthenticatedService>,

    channel: Channel,
    tokens: TokenSource,
    cfg: Config,
}

impl Client {
    pub async fn dial(cfg: Config) -> Result<Self> {
        let tokens = TokenSource::new(cfg.access_token.clone());
        let channel = connect_channel(&cfg).await?;
        let interceptor = tokens.interceptor();
        let mut client = Self {
            auth: AuthServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            space: SpaceServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            domain: DomainServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            template: TemplateServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            session: SessionServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            transaction: TransactionServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            graph: GraphServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            blob: BlobServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            query: QueryServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            import_export: ImportExportServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            metadata: MetadataCatalogServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            semantic: SemanticServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            change_stream: ChangeStreamServiceClient::with_interceptor(channel.clone(), interceptor),
            channel,
            tokens,
            cfg,
        };

        if !client.cfg.username.is_empty() || !client.cfg.password.is_empty() {
            let username = client.cfg.username.clone();
            let password = client.cfg.password.clone();
            client.login(username, password).await?;
        }

        Ok(client)
    }

    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    pub fn access_token(&self) -> String {
        self.tokens.token()
    }

    pub fn set_access_token(&self, token: impl Into<String>) {
        self.tokens.set(token);
    }

    pub async fn login(&mut self, username: impl Into<String>, password: impl Into<String>) -> Result<LoginResponse> {
        let req = self.request(LoginRequest {
            username: username.into(),
            password: password.into(),
            client: Some(self.client_info()),
        });
        let res = self.auth.login(req).await?.into_inner();
        self.set_access_token(res.access_token.clone());
        Ok(res)
    }

    pub async fn refresh(&mut self, refresh_token: Option<String>) -> Result<RefreshResponse> {
        let req = self.auth_request(RefreshRequest { refresh_token, client: Some(self.client_info()) });
        let res = self.auth.refresh(req).await?.into_inner();
        self.set_access_token(res.access_token.clone());
        Ok(res)
    }

    pub async fn who_am_i(&mut self) -> Result<PrincipalInfo> {
        let res = self.auth.who_am_i(self.auth_request(WhoAmIRequest {})).await?.into_inner();
        let principal = res.principal.unwrap_or_else(|| AuthPrincipal { user_id: String::new(), username: String::new() });
        Ok(PrincipalInfo { user_id: principal.user_id, username: principal.username })
    }

    pub(crate) fn request<T>(&self, message: T) -> Request<T> {
        let mut request = Request::new(message);
        if let Some(timeout) = self.cfg.call_timeout {
            request.set_timeout(timeout);
        }
        request
    }

    pub(crate) fn auth_request<T>(&self, message: T) -> Request<T> {
        self.request(message)
    }

    fn client_info(&self) -> ClientInfo {
        ClientInfo {
            name: non_empty(&self.cfg.client_name, "mycel-rust-sdk"),
            version: self.cfg.client_version.clone(),
            platform: non_empty(&self.cfg.platform, "rust"),
            device_label: self.cfg.device_label.clone(),
        }
    }
}

fn non_empty(value: &str, default: &str) -> String {
    let value = value.trim();
    if value.is_empty() { default.to_string() } else { value.to_string() }
}

impl From<tonic::Status> for PrincipalInfo {
    fn from(_: tonic::Status) -> Self {
        Self { user_id: String::new(), username: String::new() }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Message(value)
    }
}

pub mod graph;
pub mod session;
