use mycel_sdk::proto::admin::v1::{
    get_semantic_provider_budget_status_request, GetSemanticProviderBudgetStatusRequest,
    SemanticProviderBatchingMode, SemanticProviderBudgetBindingSelector,
    SemanticProviderBudgetPolicy,
};

#[test]
fn semantic_provider_budget_contract_constructs() {
    let policy = SemanticProviderBudgetPolicy {
        correlation_id: "managed-openai".to_string(),
        max_requests: Some(128),
        max_retry_attempts: Some(128),
        max_retries_per_request: Some(1),
        max_concurrent_provider_calls: Some(2),
        batching_mode: Some(SemanticProviderBatchingMode::Single as i32),
        max_batch_items: Some(1),
        max_input_tokens_per_request: Some(8192),
        max_input_tokens: Some(20_000),
        ..Default::default()
    };
    assert_eq!(policy.max_requests, Some(128));

    let request = GetSemanticProviderBudgetStatusRequest {
        space_id: "space".to_string(),
        selector: Some(
            get_semantic_provider_budget_status_request::Selector::CurrentBinding(
                SemanticProviderBudgetBindingSelector {
                    semantic_rule_id: "rule".to_string(),
                    embedding_binding_key: "search".to_string(),
                },
            ),
        ),
    };
    assert!(matches!(
        request.selector,
        Some(get_semantic_provider_budget_status_request::Selector::CurrentBinding(_))
    ));
}
