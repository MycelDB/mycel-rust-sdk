use mycel_sdk::proto::client::v1::{
    FilterOperator, HybridFusionStrategy, HybridSearchOptions, LexicalSearchOptions,
    PropertyFilter, SearchFilters, SearchMode, SearchRequest, SemanticSearchOptions,
};

#[test]
fn hybrid_search_request_constructs() {
    let request = SearchRequest {
        space_id: "space".to_string(),
        domain_id: "domain".to_string(),
        mode: SearchMode::Hybrid as i32,
        query: "raft recovery".to_string(),
        filters: Some(SearchFilters {
            node_labels: vec!["Note".to_string()],
            properties: vec![PropertyFilter {
                path: "tags".to_string(),
                operator: FilterOperator::Contains as i32,
                values: vec!["k3s".to_string()],
            }],
            node_ids: vec![],
        }),
        page_size: 20,
        page_token: String::new(),
        include_diagnostics: true,
        allow_stale: false,
        max_revision_lag: 0,
        hybrid: Some(HybridSearchOptions {
            lexical_weight: 0.6,
            semantic_weight: 0.4,
            fusion_strategy: HybridFusionStrategy::WeightedReciprocalRank as i32,
            require_both: false,
        }),
        semantic: Some(SemanticSearchOptions {
            semantic_rule_id: None,
            embedding_binding_key: None,
            min_score: Some(0.2),
            candidate_count: 100,
        }),
        lexical: Some(LexicalSearchOptions {
            candidate_count: 100,
        }),
    };

    assert_eq!(request.mode, SearchMode::Hybrid as i32);
    assert_eq!(request.hybrid.unwrap().lexical_weight, 0.6);
    assert_eq!(
        request.filters.unwrap().properties[0].operator,
        FilterOperator::Contains as i32
    );
}
