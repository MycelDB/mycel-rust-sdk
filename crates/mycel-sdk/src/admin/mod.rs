use mycel_proto::admin::v1::{
    admin_auth_service_client::AdminAuthServiceClient,
    admin_backup_service_client::AdminBackupServiceClient,
    admin_domain_service_client::AdminDomainServiceClient,
    admin_inference_service_client::AdminInferenceServiceClient,
    admin_operator_service_client::AdminOperatorServiceClient,
    admin_semantic_maintenance_service_client::AdminSemanticMaintenanceServiceClient,
    admin_semantic_migration_service_client::AdminSemanticMigrationServiceClient,
    admin_semantic_service_client::AdminSemanticServiceClient,
    admin_space_service_client::AdminSpaceServiceClient,
    admin_user_service_client::AdminUserServiceClient, AdminDomainServiceGetDomainRequest,
    AdminSpaceServiceListSpacesRequest, BackupPolicy, CreateSpaceRequest, CreateUserRequest,
    DeleteBackupRequest, DeleteBackupResponse, FindUserRequest, GetBackupPolicyRequest,
    GetBackupStatusRequest, GetBackupStatusResponse, ListBackupsRequest, ListBackupsResponse,
    LoginOperatorRequest, LoginOperatorResponse, Operator, OperatorClientInfo,
    TriggerBackupRequest, TriggerBackupResponse, UpdateBackupPolicyRequest, User, WhoAmIRequest,
};
use tonic::{service::interceptor::InterceptedService, transport::Channel, Code, Request};

use crate::{
    auth::{AuthInterceptor, TokenSource},
    config::Config,
    error::{Error, Result},
    transport::connect_channel,
};

pub type AuthenticatedService = InterceptedService<Channel, AuthInterceptor>;

#[derive(Debug, Clone)]
pub struct OperatorInfo {
    pub operator_id: String,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
    pub state: String,
}

#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub space_id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DomainInfo {
    pub space_id: String,
    pub domain_id: String,
    pub key: String,
    pub name: String,
    pub default: bool,
    pub system: bool,
}

#[derive(Clone)]
pub struct AdminClient {
    pub auth: AdminAuthServiceClient<AuthenticatedService>,
    pub operators: AdminOperatorServiceClient<AuthenticatedService>,
    pub users: AdminUserServiceClient<AuthenticatedService>,
    pub spaces: AdminSpaceServiceClient<AuthenticatedService>,
    pub domains: AdminDomainServiceClient<AuthenticatedService>,
    pub semantic: AdminSemanticServiceClient<AuthenticatedService>,
    pub semantic_maintenance: AdminSemanticMaintenanceServiceClient<AuthenticatedService>,
    pub semantic_migration: AdminSemanticMigrationServiceClient<AuthenticatedService>,
    pub inference: AdminInferenceServiceClient<AuthenticatedService>,
    pub backup: AdminBackupServiceClient<AuthenticatedService>,

    channel: Channel,
    tokens: TokenSource,
    cfg: Config,
}

impl AdminClient {
    pub async fn dial(cfg: Config) -> Result<Self> {
        let tokens = TokenSource::new(cfg.access_token.clone());
        let channel = connect_channel(&cfg).await?;
        let interceptor = tokens.interceptor();
        let mut client = Self {
            auth: AdminAuthServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            operators: AdminOperatorServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            users: AdminUserServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            spaces: AdminSpaceServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            domains: AdminDomainServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            semantic: AdminSemanticServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            semantic_maintenance: AdminSemanticMaintenanceServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            semantic_migration: AdminSemanticMigrationServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            inference: AdminInferenceServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            backup: AdminBackupServiceClient::with_interceptor(channel.clone(), interceptor),
            channel,
            tokens,
            cfg,
        };

        if !client.cfg.username.is_empty() || !client.cfg.password.is_empty() {
            let username = client.cfg.username.clone();
            let password = client.cfg.password.clone();
            client.login_operator(username, password).await?;
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

    pub async fn login_operator(
        &mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<LoginOperatorResponse> {
        let req = self.request(LoginOperatorRequest {
            username: username.into(),
            password: password.into(),
            client: Some(self.operator_client_info()),
        });
        let res = self.auth.login_operator(req).await?.into_inner();
        self.set_access_token(res.access_token.clone());
        Ok(res)
    }

    pub async fn who_am_i(&mut self) -> Result<OperatorInfo> {
        let res = self
            .auth
            .who_am_i(self.auth_request(WhoAmIRequest {}))
            .await?
            .into_inner();
        Ok(operator_info(res.operator.unwrap_or_default()))
    }

    pub async fn find_user(&mut self, username: impl Into<String>) -> Result<UserInfo> {
        let res = self
            .users
            .find_user(self.auth_request(FindUserRequest {
                username: username.into().trim().to_string(),
            }))
            .await?
            .into_inner();
        Ok(user_info(res.user.unwrap_or_default()))
    }

    pub async fn ensure_user(
        &mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<UserInfo> {
        let username = username.into().trim().to_string();
        if username.is_empty() {
            return Err(Error::Message("username is required".into()));
        }
        match self.find_user(username.clone()).await {
            Ok(user) => Ok(user),
            Err(Error::Status(status)) if status.code() == Code::NotFound => {
                let res = self
                    .users
                    .create_user(self.auth_request(CreateUserRequest {
                        username,
                        password: Some(password.into()),
                        disabled: false,
                    }))
                    .await?
                    .into_inner();
                Ok(user_info(res.user.unwrap_or_default()))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn find_space_by_name(&mut self, name: impl Into<String>) -> Result<SpaceInfo> {
        let name = name.into().trim().to_string();
        if name.is_empty() {
            return Err(Error::Message("space name is required".into()));
        }

        let mut token = String::new();
        let mut found: Option<SpaceInfo> = None;
        loop {
            let res = self
                .spaces
                .list_spaces(self.auth_request(AdminSpaceServiceListSpacesRequest {
                    page_size: 100,
                    page_token: token,
                    include_archived: false,
                }))
                .await?
                .into_inner();
            for sp in res.spaces {
                if sp.name != name {
                    continue;
                }
                let info = SpaceInfo {
                    space_id: sp.space_id,
                    name: sp.name,
                };
                if found.is_some() {
                    return Err(Error::Message(format!("multiple spaces named {name:?}")));
                }
                found = Some(info);
            }
            if res.next_page_token.is_empty() {
                break;
            }
            token = res.next_page_token;
        }

        found.ok_or_else(|| tonic::Status::not_found(format!("space {name:?} not found")).into())
    }

    pub async fn ensure_space(
        &mut self,
        name: impl Into<String>,
        owner_username: impl Into<String>,
        default_domain_key: impl Into<String>,
        default_domain_name: impl Into<String>,
    ) -> Result<(SpaceInfo, DomainInfo)> {
        let name = name.into();
        let owner_username = owner_username.into();
        let default_domain_key = non_empty(&default_domain_key.into(), "default");
        let default_domain_name = non_empty(&default_domain_name.into(), &default_domain_key);

        match self.find_space_by_name(name.clone()).await {
            Ok(space) => {
                let domain = self
                    .get_domain(space.space_id.clone(), default_domain_key)
                    .await?;
                Ok((space, domain))
            }
            Err(Error::Status(status)) if status.code() == Code::NotFound => {
                let res = self
                    .spaces
                    .create_space(self.auth_request(CreateSpaceRequest {
                        name: name.trim().to_string(),
                        owner_user_id: String::new(),
                        owner_username: owner_username.trim().to_string(),
                        default_domain_key: default_domain_key.clone(),
                        default_domain_name: default_domain_name.clone(),
                    }))
                    .await?
                    .into_inner();
                let space = res.space.ok_or_else(|| {
                    Error::Message("create space response did not include a space".into())
                })?;
                let sp = SpaceInfo {
                    space_id: space.space_id,
                    name: space.name,
                };
                let domain = DomainInfo {
                    space_id: sp.space_id.clone(),
                    domain_id: res.default_domain_id,
                    key: default_domain_key,
                    name: default_domain_name,
                    default: true,
                    system: false,
                };
                Ok((sp, domain))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_domain(
        &mut self,
        space_id: impl Into<String>,
        domain_ref: impl Into<String>,
    ) -> Result<DomainInfo> {
        let res = self
            .domains
            .get_domain(self.auth_request(AdminDomainServiceGetDomainRequest {
                space_id: space_id.into(),
                domain_ref: domain_ref.into(),
            }))
            .await?
            .into_inner();
        let d = res
            .domain
            .ok_or_else(|| Error::Message("get domain response did not include a domain".into()))?;
        Ok(DomainInfo {
            space_id: d.space_id,
            domain_id: d.domain_id,
            key: d.key,
            name: d.name,
            default: d.default,
            system: d.system,
        })
    }

    pub async fn get_backup_policy(&mut self) -> Result<BackupPolicy> {
        let res = self
            .backup
            .get_backup_policy(self.auth_request(GetBackupPolicyRequest {}))
            .await?
            .into_inner();
        Ok(res.policy.unwrap_or_default())
    }

    pub async fn update_backup_policy(&mut self, policy: BackupPolicy) -> Result<BackupPolicy> {
        let res = self
            .backup
            .update_backup_policy(self.auth_request(UpdateBackupPolicyRequest {
                policy: Some(policy),
            }))
            .await?
            .into_inner();
        Ok(res.policy.unwrap_or_default())
    }

    pub async fn trigger_backup(
        &mut self,
        reason: impl Into<String>,
    ) -> Result<TriggerBackupResponse> {
        let res = self
            .backup
            .trigger_backup(self.auth_request(TriggerBackupRequest {
                reason: reason.into(),
            }))
            .await?
            .into_inner();
        Ok(res)
    }

    pub async fn get_backup_status(&mut self) -> Result<GetBackupStatusResponse> {
        let res = self
            .backup
            .get_backup_status(self.auth_request(GetBackupStatusRequest {}))
            .await?
            .into_inner();
        Ok(res)
    }

    pub async fn list_backups(
        &mut self,
        page_size: i32,
        page_token: impl Into<String>,
    ) -> Result<ListBackupsResponse> {
        let res = self
            .backup
            .list_backups(self.auth_request(ListBackupsRequest {
                page_size,
                page_token: page_token.into(),
            }))
            .await?
            .into_inner();
        Ok(res)
    }

    pub async fn delete_backup(
        &mut self,
        backup_id: impl Into<String>,
    ) -> Result<DeleteBackupResponse> {
        let res = self
            .backup
            .delete_backup(self.auth_request(DeleteBackupRequest {
                backup_id: backup_id.into(),
            }))
            .await?
            .into_inner();
        Ok(res)
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

    fn operator_client_info(&self) -> OperatorClientInfo {
        OperatorClientInfo {
            name: non_empty(&self.cfg.client_name, "mycel-rust-sdk"),
            version: self.cfg.client_version.clone(),
            platform: non_empty(&self.cfg.platform, "rust"),
            device_label: self.cfg.device_label.clone(),
        }
    }
}

fn operator_info(op: Operator) -> OperatorInfo {
    OperatorInfo {
        operator_id: op.operator_id,
        username: op.username,
    }
}

fn user_info(user: User) -> UserInfo {
    let state = user.state().as_str_name().to_string();
    UserInfo {
        user_id: user.user_id,
        username: user.username,
        state,
    }
}

fn non_empty(value: &str, default: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        default.to_string()
    } else {
        value.to_string()
    }
}
