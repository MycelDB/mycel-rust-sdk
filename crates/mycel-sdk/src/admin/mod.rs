use std::{sync::Arc, time::SystemTime};

use mycel_proto::admin::v1::find_principal_request;
use mycel_proto::admin::v1::{
    admin_automation_service_client::AdminAutomationServiceClient,
    admin_backup_service_client::AdminBackupServiceClient,
    admin_cluster_service_client::AdminClusterServiceClient,
    admin_domain_service_client::AdminDomainServiceClient,
    admin_inference_catalog_service_client::AdminInferenceCatalogServiceClient,
    admin_intelligence_access_credential_service_client::AdminIntelligenceAccessCredentialServiceClient,
    admin_intelligence_access_grant_service_client::AdminIntelligenceAccessGrantServiceClient,
    admin_intelligence_access_policy_service_client::AdminIntelligenceAccessPolicyServiceClient,
    admin_intelligence_access_profile_service_client::AdminIntelligenceAccessProfileServiceClient,
    admin_intelligence_access_usage_service_client::AdminIntelligenceAccessUsageServiceClient,
    admin_principal_service_client::AdminPrincipalServiceClient,
    admin_schema_service_client::AdminSchemaServiceClient,
    admin_semantic_maintenance_service_client::AdminSemanticMaintenanceServiceClient,
    admin_semantic_migration_service_client::AdminSemanticMigrationServiceClient,
    admin_semantic_service_client::AdminSemanticServiceClient,
    admin_space_service_client::AdminSpaceServiceClient, AdminDomainServiceGetDomainRequest,
    AdminSpaceServiceListSpacesRequest, BackupArchiveFormat, BackupPolicy, CreatePrincipalRequest,
    CreateSpaceRequest, DeleteBackupRequest, DeleteBackupResponse, FindPrincipalRequest,
    GetBackupPolicyRequest, GetBackupStatusRequest, GetBackupStatusResponse,
    GetClusterBackupStatusRequest, GetClusterBackupStatusResponse, ListBackupsRequest,
    ListBackupsResponse, ListClusterBackupsRequest, ListClusterBackupsResponse, Principal,
    PrincipalCapabilityGrant, PrincipalRoleGrant, PrincipalState,
    SetPrincipalCapabilitiesForScopeRequest, SetPrincipalRolesForScopeRequest,
    TriggerBackupRequest, TriggerBackupResponse, TriggerClusterBackupRequest,
    TriggerClusterBackupResponse, UpdateBackupPolicyRequest, ValidateClusterBackupSetRequest,
    ValidateClusterBackupSetResponse,
};
use mycel_proto::common::v1::{
    auth_service_client::AuthServiceClient, AccessScope, AccessScopeType, Capability, ClientInfo,
    GetMyAccessRequest, GetMyAccessResponse, LoginRequest, LoginResponse, LogoutRequest,
    LogoutResponse, PrincipalType, RefreshRequest, RefreshResponse, WhoAmIRequest,
};
use tokio::sync::Mutex;
use tonic::{service::interceptor::InterceptedService, transport::Channel, Code, Request};

use crate::{
    auth::{is_expired_unauthenticated, timestamp_to_system_time, AuthInterceptor, TokenSource},
    config::Config,
    error::{Error, Result},
    transport::connect_channel,
};

pub type AuthenticatedService = InterceptedService<Channel, AuthInterceptor>;

macro_rules! admin_call_with_refresh {
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

#[derive(Debug, Clone)]
pub struct PrincipalAdminInfo {
    pub principal_id: String,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
    pub principal_id: String,
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

#[derive(Debug, Clone)]
pub struct PrincipalRoleGrantInfo {
    pub grant_id: String,
    pub principal_id: String,
    pub role: String,
    pub scope_type: String,
    pub space_id: String,
    pub domain_id: String,
}

#[derive(Debug, Clone)]
pub struct PrincipalCapabilityGrantInfo {
    pub grant_id: String,
    pub principal_id: String,
    pub capability: String,
    pub scope_type: String,
    pub space_id: String,
    pub domain_id: String,
}

#[derive(Clone)]
pub struct AdminClient {
    pub auth: AuthServiceClient<AuthenticatedService>,
    pub principals: AdminPrincipalServiceClient<AuthenticatedService>,
    pub spaces: AdminSpaceServiceClient<AuthenticatedService>,
    pub domains: AdminDomainServiceClient<AuthenticatedService>,
    pub semantic: AdminSemanticServiceClient<AuthenticatedService>,
    pub semantic_maintenance: AdminSemanticMaintenanceServiceClient<AuthenticatedService>,
    pub semantic_migration: AdminSemanticMigrationServiceClient<AuthenticatedService>,
    pub inference_catalog: AdminInferenceCatalogServiceClient<AuthenticatedService>,
    pub inference_profiles: AdminIntelligenceAccessProfileServiceClient<AuthenticatedService>,
    pub inference_credentials: AdminIntelligenceAccessCredentialServiceClient<AuthenticatedService>,
    pub inference_grants: AdminIntelligenceAccessGrantServiceClient<AuthenticatedService>,
    pub inference_policies: AdminIntelligenceAccessPolicyServiceClient<AuthenticatedService>,
    pub inference_usage: AdminIntelligenceAccessUsageServiceClient<AuthenticatedService>,
    pub backup: AdminBackupServiceClient<AuthenticatedService>,
    pub schema: AdminSchemaServiceClient<AuthenticatedService>,
    pub automation: AdminAutomationServiceClient<AuthenticatedService>,
    pub cluster: AdminClusterServiceClient<AuthenticatedService>,

    channel: Channel,
    tokens: TokenSource,
    refresh_lock: Arc<Mutex<()>>,
    cfg: Config,
}

impl AdminClient {
    pub async fn dial(cfg: Config) -> Result<Self> {
        let tokens = TokenSource::from_config(&cfg);
        let channel = connect_channel(&cfg).await?;
        let interceptor = tokens.interceptor();
        let mut client = Self {
            auth: AuthServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            principals: AdminPrincipalServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
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
            inference_catalog: AdminInferenceCatalogServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            inference_profiles: AdminIntelligenceAccessProfileServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            inference_credentials: AdminIntelligenceAccessCredentialServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            inference_grants: AdminIntelligenceAccessGrantServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            inference_policies: AdminIntelligenceAccessPolicyServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            inference_usage: AdminIntelligenceAccessUsageServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            backup: AdminBackupServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            schema: AdminSchemaServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            automation: AdminAutomationServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
            cluster: AdminClusterServiceClient::with_interceptor(channel.clone(), interceptor),
            channel,
            tokens,
            refresh_lock: Arc::new(Mutex::new(())),
            cfg,
        };

        if !client.cfg.username.is_empty() || !client.cfg.password.is_empty() {
            let username = client.cfg.username.clone();
            let password = client.cfg.password.clone();
            client.login_principal(username, password).await?;
        }

        Ok(client)
    }

    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    pub fn access_token(&self) -> String {
        self.tokens.token()
    }

    pub fn refresh_token(&self) -> String {
        self.tokens.refresh_token()
    }

    pub fn access_token_expire_time(&self) -> Option<SystemTime> {
        self.tokens.access_token_expire_time()
    }

    pub fn set_access_token(&self, token: impl Into<String>) {
        self.tokens.set(token);
    }

    pub fn set_refresh_token(&self, token: impl Into<String>) {
        self.tokens.set_refresh_token(token);
    }

    pub fn set_auth_tokens(
        &self,
        access_token: impl Into<String>,
        expire_time: Option<SystemTime>,
        refresh_token: impl Into<String>,
    ) {
        self.tokens
            .set_tokens(access_token, expire_time, refresh_token);
    }

    pub async fn login_principal(
        &mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<LoginResponse> {
        let req = self.request(LoginRequest {
            username: username.into(),
            password: password.into(),
            client: Some(self.client_info()),
        });
        let res = self.auth.login(req).await?.into_inner();
        self.set_auth_tokens(
            res.access_token.clone(),
            timestamp_to_system_time(res.access_token_expire_time.clone()),
            res.refresh_token.clone().unwrap_or_default(),
        );
        Ok(res)
    }

    pub async fn refresh_principal(
        &mut self,
        refresh_token: Option<String>,
    ) -> Result<RefreshResponse> {
        let refresh_token = refresh_token.or_else(|| {
            let token = self.refresh_token();
            (!token.is_empty()).then_some(token)
        });
        let req = self.auth_request(RefreshRequest {
            refresh_token,
            client: Some(self.client_info()),
        });
        let res = self.auth.refresh(req).await?.into_inner();
        self.set_auth_tokens(
            res.access_token.clone(),
            timestamp_to_system_time(res.access_token_expire_time.clone()),
            res.refresh_token.clone().unwrap_or_default(),
        );
        Ok(res)
    }

    pub async fn logout_principal(
        &mut self,
        auth_session_id: Option<String>,
    ) -> Result<LogoutResponse> {
        let req = self.auth_request(LogoutRequest {
            auth_session_id: auth_session_id.clone(),
        });
        let res = self.auth.logout(req).await?.into_inner();
        if auth_session_id.is_none() {
            self.tokens.clear();
        }
        Ok(res)
    }

    pub async fn who_am_i(&mut self) -> Result<PrincipalAdminInfo> {
        let res = admin_call_with_refresh!(
            self,
            self.auth.who_am_i(self.auth_request(WhoAmIRequest {})),
            self.auth.who_am_i(self.auth_request(WhoAmIRequest {}))
        )?
        .into_inner();
        Ok(principal_admin_info(res.principal.unwrap_or_default()))
    }

    pub async fn get_my_access(
        &mut self,
        scope: Option<AccessScope>,
    ) -> Result<GetMyAccessResponse> {
        let res = admin_call_with_refresh!(
            self,
            self.auth
                .get_my_access(self.auth_request(GetMyAccessRequest {
                    scope: scope.clone()
                })),
            self.auth
                .get_my_access(self.auth_request(GetMyAccessRequest { scope }))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn find_user(&mut self, username: impl Into<String>) -> Result<UserInfo> {
        let username = username.into().trim().to_string();
        let res = admin_call_with_refresh!(
            self,
            self.principals
                .find_principal(self.auth_request(FindPrincipalRequest {
                    lookup: Some(find_principal_request::Lookup::Username(username.clone())),
                })),
            self.principals
                .find_principal(self.auth_request(FindPrincipalRequest {
                    lookup: Some(find_principal_request::Lookup::Username(username)),
                }))
        )?
        .into_inner();
        Ok(user_info(res.principal.unwrap_or_default()))
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
                let password = password.into();
                let res = admin_call_with_refresh!(
                    self,
                    self.principals
                        .create_principal(self.auth_request(CreatePrincipalRequest {
                            username: username.clone(),
                            password: Some(password.clone()),
                            r#type: PrincipalType::Human as i32,
                            login_enabled: true,
                            ..Default::default()
                        })),
                    self.principals
                        .create_principal(self.auth_request(CreatePrincipalRequest {
                            username,
                            password: Some(password),
                            r#type: PrincipalType::Human as i32,
                            login_enabled: true,
                            ..Default::default()
                        }))
                )?
                .into_inner();
                Ok(user_info(res.principal.unwrap_or_default()))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn set_principal_roles_for_scope(
        &mut self,
        principal_id: impl Into<String>,
        scope_type: impl Into<String>,
        space_id: impl Into<String>,
        domain_id: impl Into<String>,
        roles: Vec<String>,
        reason: impl Into<String>,
    ) -> Result<Vec<PrincipalRoleGrantInfo>> {
        let principal_id = principal_id.into().trim().to_string();
        if principal_id.is_empty() {
            return Err(Error::Message("principal id is required".into()));
        }
        let scope = Some(sdk_access_scope(scope_type, space_id, domain_id));
        let reason = reason.into();
        let res = admin_call_with_refresh!(
            self,
            self.principals
                .set_principal_roles_for_scope(self.auth_request(
                    SetPrincipalRolesForScopeRequest {
                        principal_id: principal_id.clone(),
                        scope: scope.clone(),
                        roles: roles.clone(),
                        reason: reason.clone(),
                    }
                )),
            self.principals
                .set_principal_roles_for_scope(self.auth_request(
                    SetPrincipalRolesForScopeRequest {
                        principal_id,
                        scope,
                        roles,
                        reason,
                    }
                ))
        )?
        .into_inner();
        Ok(res
            .grants
            .into_iter()
            .map(principal_role_grant_info)
            .collect())
    }

    pub async fn set_principal_capabilities_for_scope(
        &mut self,
        principal_id: impl Into<String>,
        scope_type: impl Into<String>,
        space_id: impl Into<String>,
        domain_id: impl Into<String>,
        capabilities: Vec<String>,
        reason: impl Into<String>,
    ) -> Result<Vec<PrincipalCapabilityGrantInfo>> {
        let principal_id = principal_id.into().trim().to_string();
        if principal_id.is_empty() {
            return Err(Error::Message("principal id is required".into()));
        }
        let parsed = capabilities
            .iter()
            .map(|capability| sdk_capability(capability))
            .collect::<Result<Vec<_>>>()?;
        let scope = Some(sdk_access_scope(scope_type, space_id, domain_id));
        let reason = reason.into();
        let res = admin_call_with_refresh!(
            self,
            self.principals
                .set_principal_capabilities_for_scope(self.auth_request(
                    SetPrincipalCapabilitiesForScopeRequest {
                        principal_id: principal_id.clone(),
                        scope: scope.clone(),
                        capabilities: parsed.clone().into_iter().map(|cap| cap as i32).collect(),
                        reason: reason.clone(),
                    }
                )),
            self.principals
                .set_principal_capabilities_for_scope(self.auth_request(
                    SetPrincipalCapabilitiesForScopeRequest {
                        principal_id,
                        scope,
                        capabilities: parsed.into_iter().map(|cap| cap as i32).collect(),
                        reason,
                    }
                ))
        )?
        .into_inner();
        Ok(res
            .grants
            .into_iter()
            .map(principal_capability_grant_info)
            .collect())
    }

    pub async fn find_space_by_name(&mut self, name: impl Into<String>) -> Result<SpaceInfo> {
        let name = name.into().trim().to_string();
        if name.is_empty() {
            return Err(Error::Message("space name is required".into()));
        }

        let mut token = String::new();
        let mut found: Option<SpaceInfo> = None;
        loop {
            let res = admin_call_with_refresh!(
                self,
                self.spaces
                    .list_spaces(self.auth_request(AdminSpaceServiceListSpacesRequest {
                        page_size: 100,
                        page_token: token.clone(),
                        include_archived: false,
                    })),
                self.spaces
                    .list_spaces(self.auth_request(AdminSpaceServiceListSpacesRequest {
                        page_size: 100,
                        page_token: token.clone(),
                        include_archived: false,
                    }))
            )?
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
                let res = admin_call_with_refresh!(
                    self,
                    self.spaces
                        .create_space(self.auth_request(CreateSpaceRequest {
                            name: name.trim().to_string(),
                            owner_principal_id: String::new(),
                            owner_username: owner_username.trim().to_string(),
                            default_domain_key: default_domain_key.clone(),
                            default_domain_name: default_domain_name.clone(),
                        })),
                    self.spaces
                        .create_space(self.auth_request(CreateSpaceRequest {
                            name: name.trim().to_string(),
                            owner_principal_id: String::new(),
                            owner_username: owner_username.trim().to_string(),
                            default_domain_key: default_domain_key.clone(),
                            default_domain_name: default_domain_name.clone(),
                        }))
                )?
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
        let space_id = space_id.into();
        let domain_ref = domain_ref.into();
        let res = admin_call_with_refresh!(
            self,
            self.domains
                .get_domain(self.auth_request(AdminDomainServiceGetDomainRequest {
                    space_id: space_id.clone(),
                    domain_ref: domain_ref.clone(),
                })),
            self.domains
                .get_domain(self.auth_request(AdminDomainServiceGetDomainRequest {
                    space_id,
                    domain_ref,
                }))
        )?
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
        let res = admin_call_with_refresh!(
            self,
            self.backup
                .get_backup_policy(self.auth_request(GetBackupPolicyRequest {})),
            self.backup
                .get_backup_policy(self.auth_request(GetBackupPolicyRequest {}))
        )?
        .into_inner();
        Ok(res.policy.unwrap_or_default())
    }

    pub async fn update_backup_policy(&mut self, policy: BackupPolicy) -> Result<BackupPolicy> {
        let res = admin_call_with_refresh!(
            self,
            self.backup
                .update_backup_policy(self.auth_request(UpdateBackupPolicyRequest {
                    policy: Some(policy.clone()),
                })),
            self.backup
                .update_backup_policy(self.auth_request(UpdateBackupPolicyRequest {
                    policy: Some(policy),
                }))
        )?
        .into_inner();
        Ok(res.policy.unwrap_or_default())
    }

    pub async fn trigger_backup(
        &mut self,
        reason: impl Into<String>,
    ) -> Result<TriggerBackupResponse> {
        let reason = reason.into();
        let res = admin_call_with_refresh!(
            self,
            self.backup
                .trigger_backup(self.auth_request(TriggerBackupRequest {
                    reason: reason.clone(),
                })),
            self.backup
                .trigger_backup(self.auth_request(TriggerBackupRequest { reason }))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn get_backup_status(&mut self) -> Result<GetBackupStatusResponse> {
        let res = admin_call_with_refresh!(
            self,
            self.backup
                .get_backup_status(self.auth_request(GetBackupStatusRequest {})),
            self.backup
                .get_backup_status(self.auth_request(GetBackupStatusRequest {}))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn list_backups(
        &mut self,
        page_size: i32,
        page_token: impl Into<String>,
    ) -> Result<ListBackupsResponse> {
        let page_token = page_token.into();
        let res = admin_call_with_refresh!(
            self,
            self.backup
                .list_backups(self.auth_request(ListBackupsRequest {
                    page_size,
                    page_token: page_token.clone(),
                })),
            self.backup
                .list_backups(self.auth_request(ListBackupsRequest {
                    page_size,
                    page_token,
                }))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn delete_backup(
        &mut self,
        backup_id: impl Into<String>,
    ) -> Result<DeleteBackupResponse> {
        let backup_id = backup_id.into();
        let res = admin_call_with_refresh!(
            self,
            self.backup
                .delete_backup(self.auth_request(DeleteBackupRequest {
                    backup_id: backup_id.clone(),
                })),
            self.backup
                .delete_backup(self.auth_request(DeleteBackupRequest { backup_id }))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn trigger_cluster_backup(
        &mut self,
        reason: impl Into<String>,
        output_dir: impl Into<String>,
        archive_format: BackupArchiveFormat,
    ) -> Result<TriggerClusterBackupResponse> {
        let reason = reason.into();
        let output_dir = output_dir.into();
        let res = admin_call_with_refresh!(
            self,
            self.backup
                .trigger_cluster_backup(self.auth_request(TriggerClusterBackupRequest {
                    reason: reason.clone(),
                    output_dir: output_dir.clone(),
                    archive_format: archive_format as i32,
                })),
            self.backup
                .trigger_cluster_backup(self.auth_request(TriggerClusterBackupRequest {
                    reason,
                    output_dir,
                    archive_format: archive_format as i32,
                }))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn get_cluster_backup_status(
        &mut self,
        backup_set_id: impl Into<String>,
    ) -> Result<GetClusterBackupStatusResponse> {
        let backup_set_id = backup_set_id.into();
        let res = admin_call_with_refresh!(
            self,
            self.backup.get_cluster_backup_status(self.auth_request(
                GetClusterBackupStatusRequest {
                    backup_set_id: backup_set_id.clone(),
                }
            )),
            self.backup.get_cluster_backup_status(
                self.auth_request(GetClusterBackupStatusRequest { backup_set_id })
            )
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn list_cluster_backups(
        &mut self,
        page_size: i32,
        page_token: impl Into<String>,
    ) -> Result<ListClusterBackupsResponse> {
        let page_token = page_token.into();
        let res = admin_call_with_refresh!(
            self,
            self.backup
                .list_cluster_backups(self.auth_request(ListClusterBackupsRequest {
                    page_size,
                    page_token: page_token.clone(),
                })),
            self.backup
                .list_cluster_backups(self.auth_request(ListClusterBackupsRequest {
                    page_size,
                    page_token,
                }))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn validate_cluster_backup_set(
        &mut self,
        backup_set_path: impl Into<String>,
    ) -> Result<ValidateClusterBackupSetResponse> {
        let backup_set_path = backup_set_path.into();
        let res = admin_call_with_refresh!(
            self,
            self.backup.validate_cluster_backup_set(self.auth_request(
                ValidateClusterBackupSetRequest {
                    backup_set_path: backup_set_path.clone(),
                }
            )),
            self.backup.validate_cluster_backup_set(
                self.auth_request(ValidateClusterBackupSetRequest { backup_set_path })
            )
        )?
        .into_inner();
        Ok(res)
    }

    pub(crate) async fn refresh_if_needed(&mut self) -> Result<()> {
        if !self.tokens.needs_refresh(SystemTime::now()) {
            return Ok(());
        }
        let refresh_lock = self.refresh_lock.clone();
        let _guard = refresh_lock.lock().await;
        if !self.tokens.needs_refresh(SystemTime::now()) {
            return Ok(());
        }
        self.refresh_principal(None).await.map(|_| ())
    }

    pub(crate) async fn refresh_after_expired(&mut self) -> Result<()> {
        if !self.tokens.can_refresh() {
            return Ok(());
        }
        let refresh_lock = self.refresh_lock.clone();
        let _guard = refresh_lock.lock().await;
        self.refresh_principal(None).await.map(|_| ())
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

fn sdk_access_scope(
    scope_type: impl Into<String>,
    space_id: impl Into<String>,
    domain_id: impl Into<String>,
) -> AccessScope {
    let scope_type = scope_type.into().trim().to_ascii_lowercase();
    let space_id = space_id.into().trim().to_string();
    let domain_id = domain_id.into().trim().to_string();
    let typ = match scope_type.as_str() {
        "space" => AccessScopeType::Space,
        "domain" => AccessScopeType::Domain,
        "" | "system" if !domain_id.is_empty() => AccessScopeType::Domain,
        "" | "system" if !space_id.is_empty() => AccessScopeType::Space,
        _ => AccessScopeType::System,
    };
    AccessScope {
        r#type: typ as i32,
        space_id: (!space_id.is_empty()).then_some(space_id),
        domain_id: (!domain_id.is_empty()).then_some(domain_id),
    }
}

fn sdk_capability(raw: &str) -> Result<Capability> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(Error::Message("capability is required".into()));
    }
    let normalized = trimmed.replace(['.', '-'], "_").to_ascii_uppercase();
    let name = if normalized.starts_with("CAPABILITY_") {
        normalized
    } else {
        format!("CAPABILITY_{normalized}")
    };
    Capability::from_str_name(&name)
        .filter(|capability| *capability != Capability::Unspecified)
        .ok_or_else(|| Error::Message(format!("unknown capability {raw:?}")))
}

fn principal_role_grant_info(grant: PrincipalRoleGrant) -> PrincipalRoleGrantInfo {
    let scope = grant.scope.unwrap_or_default();
    PrincipalRoleGrantInfo {
        grant_id: grant.role_grant_id,
        principal_id: grant.principal_id,
        role: grant.role,
        scope_type: AccessScopeType::try_from(scope.r#type)
            .map(|typ| typ.as_str_name().to_string())
            .unwrap_or_else(|_| format!("ACCESS_SCOPE_TYPE_UNKNOWN_{}", scope.r#type)),
        space_id: scope.space_id.unwrap_or_default(),
        domain_id: scope.domain_id.unwrap_or_default(),
    }
}

fn principal_capability_grant_info(
    grant: PrincipalCapabilityGrant,
) -> PrincipalCapabilityGrantInfo {
    let scope = grant.scope.unwrap_or_default();
    PrincipalCapabilityGrantInfo {
        grant_id: grant.capability_grant_id,
        principal_id: grant.principal_id,
        capability: Capability::try_from(grant.capability)
            .map(|capability| capability.as_str_name().to_string())
            .unwrap_or_else(|_| format!("CAPABILITY_UNKNOWN_{}", grant.capability)),
        scope_type: AccessScopeType::try_from(scope.r#type)
            .map(|typ| typ.as_str_name().to_string())
            .unwrap_or_else(|_| format!("ACCESS_SCOPE_TYPE_UNKNOWN_{}", scope.r#type)),
        space_id: scope.space_id.unwrap_or_default(),
        domain_id: scope.domain_id.unwrap_or_default(),
    }
}

fn principal_admin_info(principal: mycel_proto::common::v1::AuthPrincipal) -> PrincipalAdminInfo {
    PrincipalAdminInfo {
        principal_id: principal.principal_id,
        username: principal.username,
    }
}

fn user_info(principal: Principal) -> UserInfo {
    let state = PrincipalState::try_from(principal.state)
        .unwrap_or(PrincipalState::Unspecified)
        .as_str_name()
        .to_string();
    UserInfo {
        user_id: principal.principal_id.clone(),
        principal_id: principal.principal_id,
        username: principal.username,
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
