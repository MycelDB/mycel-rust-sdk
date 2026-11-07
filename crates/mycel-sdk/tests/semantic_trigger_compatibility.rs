use std::{env, fs, path::PathBuf};

use mycel_sdk::{
    dial_admin,
    proto::admin::v1::{
        SemanticEmbeddingBinding, SemanticGenerationRule, SemanticSourceAssemblyPolicy,
        SemanticTargetSelector, SemanticTriggerEventType, SemanticTriggerPolicy,
        ValidateSemanticRuleRequest,
    },
    Config,
};
use prost::Message;
use serde_json::json;
use tonic::Code;

#[test]
fn v021_semantic_trigger_typed_wire_contract() {
    let policy = SemanticTriggerPolicy {
        events: vec![
            SemanticTriggerEventType::Changed as i32,
            SemanticTriggerEventType::NodeCreated as i32,
            SemanticTriggerEventType::NodeUpdated as i32,
            SemanticTriggerEventType::NodeDeleted as i32,
            SemanticTriggerEventType::EdgeChanged as i32,
            SemanticTriggerEventType::EdgeCreated as i32,
            SemanticTriggerEventType::EdgeUpdated as i32,
            SemanticTriggerEventType::EdgeDeleted as i32,
        ],
        labels: Vec::new(),
        debounce: String::new(),
    };

    let wire = policy.encode_to_vec();
    assert_eq!(
        wire,
        [0x0a, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
    );
    assert_eq!(
        SemanticTriggerPolicy::decode(wire.as_slice()).unwrap(),
        policy
    );
}

#[test]
fn v020_legacy_semantic_trigger_wire_fixture_decodes_as_unsupported_enum() {
    let wire = decode_hex(include_str!(
        "fixtures/semantic-trigger-v020-node-created.hex"
    ));
    let decoded = SemanticTriggerPolicy::decode(wire.as_slice()).unwrap();

    assert_eq!(decoded.events.first(), Some(&110));
    assert!(SemanticTriggerEventType::try_from(decoded.events[0]).is_err());
    assert_eq!(
        format!(
            "trigger.events[0] contains unsupported semantic trigger event value {}",
            decoded.events[0]
        ),
        "trigger.events[0] contains unsupported semantic trigger event value 110"
    );
}

#[test]
fn compatibility_manifest_v1() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.release/mycel-compatibility.json");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(path).expect("read compatibility manifest"))
            .expect("parse compatibility manifest");

    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["component"], "mycel-rust-sdk");
    assert_eq!(manifest["component_release"], "v0.21.2");
    assert_eq!(
        manifest["source_revision"],
        "9ad5057498f2e55ac2836dd0417de5c9583377e6"
    );
    assert_eq!(manifest["mycel_api_line"], "0.21");
    assert_eq!(manifest["supported_daemon_api_lines"], json!(["0.21"]));
    assert_eq!(
        manifest["incompatible_daemon_api_lines"],
        json!([{
            "line": "0.20",
            "contracts": [
                "search-channel-owned-query-v1",
                "semantic-trigger-events-enum-v1"
            ]
        }])
    );
    assert_eq!(manifest["published_at"], "2026-11-07T20:05:24Z");
}

#[tokio::test]
async fn v021_semantic_trigger_compatibility_against_daemon() {
    let space_id = env::var("MYCEL_V021_COMPAT_SPACE_ID").unwrap_or_default();
    let domain_id = env::var("MYCEL_V021_COMPAT_DOMAIN_ID").unwrap_or_default();
    if space_id.trim().is_empty() || domain_id.trim().is_empty() {
        eprintln!(
            "skipping read-only daemon check; set MYCEL_V021_COMPAT_SPACE_ID and MYCEL_V021_COMPAT_DOMAIN_ID"
        );
        return;
    }

    let mut admin = dial_admin(Config::from_env()).await.expect("dial admin");
    let rule = SemanticGenerationRule {
        space_id,
        domain_id,
        key: "sdk-v021-semantic-trigger-compatibility".to_string(),
        trigger: Some(SemanticTriggerPolicy {
            events: vec![
                SemanticTriggerEventType::NodeCreated as i32,
                SemanticTriggerEventType::NodeUpdated as i32,
                SemanticTriggerEventType::EdgeCreated as i32,
                SemanticTriggerEventType::EdgeDeleted as i32,
            ],
            ..Default::default()
        }),
        selector: Some(SemanticTargetSelector {
            mode: "node_type".to_string(),
            labels: vec!["CompatibilityFixture".to_string()],
            ..Default::default()
        }),
        source: Some(SemanticSourceAssemblyPolicy {
            mode: "self".to_string(),
            ..Default::default()
        }),
        embeddings: vec![SemanticEmbeddingBinding {
            key: "default".to_string(),
            purpose: "compatibility".to_string(),
            intelligence_profile: "compatibility-profile".to_string(),
            vector_store: "compatibility-vector-store".to_string(),
            enabled: true,
            ..Default::default()
        }],
        ..Default::default()
    };

    let validation = admin
        .semantic
        .validate_semantic_rule(ValidateSemanticRuleRequest {
            rule: Some(rule.clone()),
        })
        .await
        .expect("validate typed v0.21 semantic rule")
        .into_inner();
    assert!(
        validation.valid,
        "diagnostics = {:?}",
        validation.diagnostics
    );

    let mut legacy_equivalent = rule;
    legacy_equivalent.trigger.as_mut().unwrap().events = vec![110];
    let error = admin
        .semantic
        .validate_semantic_rule(ValidateSemanticRuleRequest {
            rule: Some(legacy_equivalent),
        })
        .await
        .expect_err("legacy-equivalent event must fail");
    assert_eq!(error.code(), Code::InvalidArgument);
    assert!(
        error
            .message()
            .contains("trigger.events[0] contains unsupported semantic trigger event value 110"),
        "error = {error}"
    );
}

fn decode_hex(value: &str) -> Vec<u8> {
    let value = value.trim();
    assert_eq!(
        value.len() % 2,
        0,
        "hex fixture must contain complete bytes"
    );
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect()
}
