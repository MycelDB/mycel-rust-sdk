use std::collections::HashMap;

use mycel_sdk::proto::admin::v1::{
    batch_usage_result, usage_scope, BatchGetUsageRequest, BatchGetUsageResponse, BatchUsageResult,
    DeleteUsageLimitRequest, DomainInclusionMode, DomainUsageScope,
    GetUsageAccountingHealthRequest, GetUsageAccountingHealthResponse, GetUsageLimitRequest,
    GetUsageRequest, GetUsageResponse, ListUsageLimitsRequest, ListUsageLimitsResponse,
    ListUsageRequest, ListUsageResponse, SpaceUsageScope, UpsertUsageLimitRequest,
    UsageAccountingHealth, UsageAccountingState, UsageAuthority, UsageLimit, UsageMetric,
    UsageRevision, UsageScope, UsageSourceRevision, UsageStatus, UsageStatusReason,
    UsageStatusSeverity, UsageUnit,
};
use mycel_sdk::AdminClient;

#[test]
fn usage_accounting_contract_types_compile() {
    let scope = UsageScope {
        scope: Some(usage_scope::Scope::Domain(DomainUsageScope {
            space_id: "space-1".to_string(),
            domain_id: "domain-1".to_string(),
        })),
    };
    let revision = UsageRevision {
        revision_token: "usage-42".to_string(),
        source_revisions: vec![UsageSourceRevision {
            source: "partition-0".to_string(),
            revision: "raft-index-10".to_string(),
        }],
    };
    let request = GetUsageRequest {
        scope: Some(scope.clone()),
        domain_inclusion: DomainInclusionMode::RetainedNonSystem as i32,
        include_diagnostic_physical: false,
        require_quota_safe: true,
        max_staleness_seconds: 60,
        minimum_revision: Some(revision.clone()),
        metric_names: vec!["blob.active_logical_bytes".to_string()],
    };
    let response = GetUsageResponse {
        snapshot: Some(mycel_sdk::proto::admin::v1::UsageSnapshot {
            scope: request.scope.clone(),
            domain_inclusion: request.domain_inclusion,
            observed_at: None,
            as_of_revision: Some(revision.clone()),
            accounting_state: UsageAccountingState::Ready as i32,
            quota_safe_complete: true,
            metrics: vec![UsageMetric {
                name: "blob.active_logical_bytes".to_string(),
                metric_version: 1,
                value: 1024,
                unit: UsageUnit::Bytes as i32,
                authority: UsageAuthority::RaftCommittedExact as i32,
                quota_safe: true,
                hard_limit_eligible: true,
                observed_at: None,
                as_of_revision: Some(revision),
                labels: HashMap::new(),
            }],
            blockers: Vec::new(),
            warnings: Vec::new(),
        }),
    };

    assert!(request.require_quota_safe);
    let snapshot = response.snapshot.unwrap();
    assert!(snapshot.quota_safe_complete);
    assert_eq!(snapshot.metrics[0].value, 1024);
}

#[test]
fn usage_accounting_batch_and_health_contract_types_compile() {
    let scope = UsageScope {
        scope: Some(usage_scope::Scope::Space(SpaceUsageScope {
            space_id: "space-1".to_string(),
        })),
    };
    let mut evidence = HashMap::new();
    evidence.insert("state".to_string(), "rebuilding".to_string());
    let status = UsageStatus {
        reason: UsageStatusReason::CountersRebuilding as i32,
        severity: UsageStatusSeverity::Blocker as i32,
        message: "usage counters are rebuilding".to_string(),
        scope: Some(scope.clone()),
        metric_name: "blob.active_count".to_string(),
        evidence,
    };
    let batch_request = BatchGetUsageRequest {
        requests: vec![GetUsageRequest {
            scope: Some(scope.clone()),
            require_quota_safe: true,
            ..Default::default()
        }],
    };
    let batch_response = BatchGetUsageResponse {
        results: vec![BatchUsageResult {
            scope: Some(scope.clone()),
            result: Some(batch_usage_result::Result::Status(status.clone())),
        }],
    };
    let health_request = GetUsageAccountingHealthRequest {
        scope: Some(scope.clone()),
        include_reconciliation: true,
        include_diagnostic_collectors: false,
    };
    let health_response = GetUsageAccountingHealthResponse {
        health: Some(UsageAccountingHealth {
            scope: Some(scope),
            observed_at: None,
            state: UsageAccountingState::Rebuilding as i32,
            as_of_revision: None,
            quota_safe_ready: false,
            blockers: vec![status],
            warnings: Vec::new(),
            rebuilding_scope_count: 1,
            stale_scope_count: 0,
            drift_scope_count: 0,
            diagnostic_collector_unavailable_count: 0,
        }),
    };
    let list_request = ListUsageRequest {
        mode: mycel_sdk::proto::admin::v1::UsageListMode::Spaces as i32,
        page_size: 100,
        ..Default::default()
    };
    let list_response = ListUsageResponse {
        snapshots: Vec::new(),
        next_page_token: String::new(),
    };

    assert!(batch_request.requests[0].require_quota_safe);
    assert!(matches!(
        batch_response.results[0].result,
        Some(batch_usage_result::Result::Status(_))
    ));
    assert!(health_request.include_reconciliation);
    assert!(!health_response.health.unwrap().quota_safe_ready);
    assert_eq!(list_request.page_size, 100);
    assert!(list_response.snapshots.is_empty());
}

#[test]
fn usage_admission_contract_types_compile() {
    let scope = UsageScope {
        scope: Some(usage_scope::Scope::Domain(DomainUsageScope {
            space_id: "space-1".to_string(),
            domain_id: "domain-1".to_string(),
        })),
    };
    let limit = UsageLimit {
        limit_id: String::new(),
        scope: Some(scope.clone()),
        domain_inclusion: DomainInclusionMode::Unspecified as i32,
        metric_name: "blob.active_logical_bytes".to_string(),
        metric_version: 1,
        hard_limit: 1024,
        enabled: true,
        policy_ref: "plan/example".to_string(),
        description: String::new(),
        created_at: None,
        updated_at: None,
    };
    let upsert = UpsertUsageLimitRequest {
        limit: Some(limit.clone()),
    };
    let list_response = ListUsageLimitsResponse {
        limits: vec![limit],
        next_page_token: String::new(),
    };

    assert!(upsert.limit.unwrap().enabled);
    assert_eq!(list_response.limits[0].hard_limit, 1024);
}

#[allow(dead_code)]
async fn admin_usage_helper_methods_compile(client: &mut AdminClient) -> mycel_sdk::Result<()> {
    let _ = client.get_usage(GetUsageRequest::default()).await?;
    let _ = client
        .batch_get_usage(BatchGetUsageRequest::default())
        .await?;
    let _ = client.list_usage(ListUsageRequest::default()).await?;
    let _ = client
        .get_usage_accounting_health(GetUsageAccountingHealthRequest::default())
        .await?;
    let _ = client
        .upsert_usage_limit(UpsertUsageLimitRequest::default())
        .await?;
    let _ = client
        .delete_usage_limit(DeleteUsageLimitRequest::default())
        .await?;
    let _ = client
        .get_usage_limit(GetUsageLimitRequest::default())
        .await?;
    let _ = client
        .list_usage_limits(ListUsageLimitsRequest::default())
        .await?;
    Ok(())
}
