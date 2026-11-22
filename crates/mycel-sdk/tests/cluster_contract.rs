use std::collections::HashMap;

use mycel_sdk::proto::admin::v1::{ClusterDerivedSubsystemReadiness, ClusterReadiness};

#[test]
fn cluster_derived_readiness_contract_types_compile() {
    let readiness = ClusterReadiness {
        client_ready: true,
        derived_ready: false,
        restored_cluster_ready: false,
        derived_subsystems: vec![ClusterDerivedSubsystemReadiness {
            subsystem: "LEXICAL_SEARCH".to_string(),
            state: "rebuild_required".to_string(),
            rebuild_required: true,
            reason: "derived_artifact_omitted_rebuild_required".to_string(),
            evidence: HashMap::from([("backup_set_id".to_string(), "backup-set-1".to_string())]),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(!readiness.derived_ready);
    assert!(!readiness.restored_cluster_ready);
    assert!(readiness.derived_subsystems[0].rebuild_required);
}
