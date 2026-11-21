use std::collections::HashMap;

use mycel_sdk::proto::admin::v1::{
    BackupRepository, BackupRepositoryProvider, BackupSetManifestSummary, ClusterBackupSetSummary,
    ClusterRestoreAuthSessionPolicy, ClusterRestoreIdentityMode, ClusterRestoreNodeMapping,
    ClusterRestoreStagingLocation, ClusterRestoreTarget, ClusterRestoreTargetMode,
    CompatibilitySummary, CreateBackupRepositoryRequest, EncryptionMode, EncryptionPolicy,
    PlanClusterRestoreRequest, S3CompatibleRepositoryConfig, SecretReference,
    SecretReferenceScheme, StartClusterRestoreRequest, ValidationOptions, ValidationSummary,
};
use mycel_sdk::AdminClient;

#[test]
fn backup_repository_contract_types_compile() {
    let request = CreateBackupRepositoryRequest {
        repository: Some(BackupRepository {
            repository_id: "repo-prod".to_string(),
            display_name: "Production backups".to_string(),
            provider: BackupRepositoryProvider::S3Compatible as i32,
            s3: Some(S3CompatibleRepositoryConfig {
                bucket: "mycel-backups".to_string(),
                prefix: "clusters/prod".to_string(),
                region: "us-east-1".to_string(),
                endpoint_uri: String::new(),
                force_path_style: false,
                disable_tls: false,
            }),
            credential_ref: Some(SecretReference {
                scheme: SecretReferenceScheme::AwsIrsa as i32,
                name: String::new(),
                namespace: String::new(),
                path: String::new(),
                keys: Default::default(),
                description: String::new(),
            }),
            encryption: Some(EncryptionPolicy {
                mode: EncryptionMode::SseKms as i32,
                key_ref: None,
                kms_key_id: "alias/mycel-backups".to_string(),
                require_bucket_default_encryption: true,
            }),
            tls_ca_ref: None,
            dev_only: false,
            create_time: String::new(),
            update_time: String::new(),
            validation: None,
        }),
    };

    assert_eq!(
        request.repository.unwrap().provider,
        BackupRepositoryProvider::S3Compatible as i32
    );
}

#[test]
fn restore_contract_types_compile() {
    let request = PlanClusterRestoreRequest {
        repository_id: "repo-prod".to_string(),
        backup_set_id: "backup-set-1".to_string(),
        target_mode: ClusterRestoreTargetMode::NewClusterBootstrap as i32,
        identity_mode: ClusterRestoreIdentityMode::PreserveBackupId as i32,
        auth_session_policy: ClusterRestoreAuthSessionPolicy::PreserveRestoredSessions as i32,
        validation_options: Some(ValidationOptions {
            include_artifact_checks: true,
            include_external_dependency_checks: false,
            include_capacity_estimates: true,
            require_complete_marker: true,
        }),
        target: Some(ClusterRestoreTarget {
            target_cluster_name: "restored-prod".to_string(),
            expected_node_count: 3,
            nodes: vec![ClusterRestoreNodeMapping {
                ordinal: 0,
                target_pvc_name: "data-myceld-0".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }),
        staging: Some(ClusterRestoreStagingLocation {
            staging_uri: "file:///restore-staging".to_string(),
            clean_before_use: true,
            ..Default::default()
        }),
    };

    assert_eq!(
        request.target_mode,
        ClusterRestoreTargetMode::NewClusterBootstrap as i32
    );
}

#[test]
fn backup_set_validation_summary_fields_compile() {
    let mut storage_versions = HashMap::new();
    storage_versions.insert("system_raft".to_string(), "1".to_string());
    let summary = ClusterBackupSetSummary {
        backup_set_id: "backup-set-1".to_string(),
        validation: Some(ValidationSummary {
            validator_version: "v1".to_string(),
            expected_node_count: 3,
            node_artifact_count: 3,
            required_artifact_count: 3,
            validated_artifact_count: 3,
            total_artifact_bytes: 1024,
            checksum_algorithm: "SHA256".to_string(),
            ..Default::default()
        }),
        compatibility: Some(CompatibilitySummary {
            required_features: vec!["checkpoint_barrier_set.v1".to_string()],
            storage_format_versions: storage_versions,
            min_restore_mycel_version: "v0.24.0".to_string(),
            supported_restore_target_modes: vec!["NEW_CLUSTER_BOOTSTRAP".to_string()],
            supported_identity_modes: vec!["PRESERVE_BACKUP_ID".to_string()],
            requires_same_topology: true,
            requires_source_fencing_for_preserved_identity: true,
            ..Default::default()
        }),
        manifest: Some(BackupSetManifestSummary {
            manifest_version: 1,
            manifest_size_bytes: 2048,
            total_artifact_bytes: 1024,
            artifact_count: 3,
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_eq!(summary.validation.unwrap().checksum_algorithm, "SHA256");
    assert!(summary.compatibility.unwrap().requires_same_topology);
    assert_eq!(summary.manifest.unwrap().artifact_count, 3);
}

#[allow(dead_code)]
async fn admin_backup_helper_methods_compile(
    client: &mut AdminClient,
    repository: BackupRepository,
) -> mycel_sdk::Result<()> {
    let _ = client.list_backup_repositories(100, "").await?;
    let _ = client.create_backup_repository(repository.clone()).await?;
    let _ = client
        .get_backup_repository(repository.repository_id.clone())
        .await?;
    let _ = client.update_backup_repository(repository.clone()).await?;
    let _ = client
        .validate_backup_repository(repository.repository_id.clone())
        .await?;
    let _ = client.list_cluster_backup_operations(100, "").await?;
    let _ = client
        .list_cluster_backup_sets(repository.repository_id.clone(), 100, "")
        .await?;
    let _ = client
        .get_cluster_backup_set(repository.repository_id.clone(), "backup-set-1")
        .await?;
    let _ = client
        .validate_repository_cluster_backup_set(repository.repository_id.clone(), "backup-set-1")
        .await?;
    let _ = client
        .delete_cluster_backup_set(repository.repository_id.clone(), "backup-set-1", "test")
        .await?;
    let _ = client
        .plan_cluster_restore(PlanClusterRestoreRequest {
            repository_id: repository.repository_id.clone(),
            backup_set_id: "backup-set-1".to_string(),
            target_mode: ClusterRestoreTargetMode::NewClusterBootstrap as i32,
            identity_mode: ClusterRestoreIdentityMode::PreserveBackupId as i32,
            auth_session_policy: ClusterRestoreAuthSessionPolicy::PreserveRestoredSessions as i32,
            validation_options: None,
            target: None,
            staging: None,
        })
        .await?;
    let _ = client
        .start_cluster_restore(StartClusterRestoreRequest {
            restore_plan_id: "restore-plan-1".to_string(),
            repository_id: repository.repository_id.clone(),
            backup_set_id: "backup-set-1".to_string(),
            target_mode: ClusterRestoreTargetMode::NewClusterBootstrap as i32,
            identity_mode: ClusterRestoreIdentityMode::PreserveBackupId as i32,
            auth_session_policy: ClusterRestoreAuthSessionPolicy::PreserveRestoredSessions as i32,
            idempotency_key: "idem-1".to_string(),
            destructive_confirmation: String::new(),
            target: None,
            staging: None,
            preconditions: None,
            plan_freshness_token: String::new(),
        })
        .await?;
    let _ = client.get_cluster_restore_status("restore-op-1").await?;
    let _ = client.list_cluster_restores(100, "").await?;
    let _ = client
        .cancel_cluster_restore("restore-op-1", "test")
        .await?;
    let _ = client
        .delete_backup_repository(repository.repository_id)
        .await?;
    Ok(())
}
