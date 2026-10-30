use mycel::client::v1::{
    SemanticEligibilityFilter, SemanticPropertyOperator, SemanticPropertyPredicate,
    SemanticSearchRequest, SemanticSearchResponse,
};
use prost_types::value;
pub use prost_types::Value;

use crate::{Client, Result};

impl Client {
    /// Executes one direct-semantic search request through the coordinated
    /// authentication-refresh boundary.
    ///
    /// Page tokens are opaque. This helper never follows or restarts pagination
    /// automatically, and an expired-token retry replays the exact same request.
    pub async fn semantic_search(
        &mut self,
        request: SemanticSearchRequest,
    ) -> Result<SemanticSearchResponse> {
        self.authenticated_call(|client| {
            let request = request.clone();
            Box::pin(async move {
                let request = client.request(request);
                client
                    .semantic
                    .semantic_search(request)
                    .await
                    .map(|response| response.into_inner())
            })
        })
        .await
    }
}

/// Builds a conjunctive direct-semantic hard-eligibility filter.
#[derive(Clone, Debug, Default)]
pub struct SemanticEligibilityFilterBuilder {
    filter: SemanticEligibilityFilter,
}

impl SemanticEligibilityFilterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn require_label(mut self, label: impl Into<String>) -> Self {
        self.filter.node_labels.push(label.into());
        self
    }

    pub fn require_node_id(mut self, node_id: impl Into<String>) -> Self {
        self.filter.node_ids.push(node_id.into());
        self
    }

    pub fn predicate(mut self, predicate: SemanticPropertyPredicate) -> Self {
        self.filter.properties.push(predicate);
        self
    }

    pub fn equal(self, key: impl Into<String>, value: Value) -> Self {
        self.predicate(semantic_equal(key, value))
    }

    pub fn not_equal(self, key: impl Into<String>, value: Value) -> Self {
        self.predicate(semantic_not_equal(key, value))
    }

    pub fn exists(self, key: impl Into<String>) -> Self {
        self.predicate(semantic_exists(key))
    }

    pub fn missing(self, key: impl Into<String>) -> Self {
        self.predicate(semantic_missing(key))
    }

    pub fn build(self) -> SemanticEligibilityFilter {
        self.filter
    }
}

/// Constructs an exact, type-sensitive equality predicate.
pub fn semantic_equal(key: impl Into<String>, value: Value) -> SemanticPropertyPredicate {
    semantic_value_predicate(key, SemanticPropertyOperator::Equal, value)
}

/// Constructs an exact, type-sensitive inequality predicate.
pub fn semantic_not_equal(key: impl Into<String>, value: Value) -> SemanticPropertyPredicate {
    semantic_value_predicate(key, SemanticPropertyOperator::NotEqual, value)
}

/// Matches a present property, including an explicit null.
pub fn semantic_exists(key: impl Into<String>) -> SemanticPropertyPredicate {
    SemanticPropertyPredicate {
        key: key.into(),
        operator: SemanticPropertyOperator::Exists as i32,
        value: None,
    }
}

/// Matches only an absent property.
pub fn semantic_missing(key: impl Into<String>) -> SemanticPropertyPredicate {
    SemanticPropertyPredicate {
        key: key.into(),
        operator: SemanticPropertyOperator::Missing as i32,
        value: None,
    }
}

/// Constructs an explicit protobuf null value for equality predicates.
pub fn null_value() -> Value {
    Value {
        kind: Some(value::Kind::NullValue(0)),
    }
}

/// Constructs a protobuf string value for equality predicates.
pub fn string_value(value: impl Into<String>) -> Value {
    Value {
        kind: Some(value::Kind::StringValue(value.into())),
    }
}

/// Constructs a protobuf numeric value for equality predicates.
pub fn number_value(value: f64) -> Value {
    Value {
        kind: Some(value::Kind::NumberValue(value)),
    }
}

/// Constructs a protobuf boolean value for equality predicates.
pub fn bool_value(value: bool) -> Value {
    Value {
        kind: Some(value::Kind::BoolValue(value)),
    }
}

fn semantic_value_predicate(
    key: impl Into<String>,
    operator: SemanticPropertyOperator,
    value: Value,
) -> SemanticPropertyPredicate {
    SemanticPropertyPredicate {
        key: key.into(),
        operator: operator as i32,
        value: Some(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn predicates_distinguish_explicit_null_from_missing() {
        let explicit_null = semantic_equal("archived_at", null_value());
        assert_eq!(
            explicit_null.operator,
            SemanticPropertyOperator::Equal as i32
        );
        assert!(matches!(
            explicit_null.value.and_then(|value| value.kind),
            Some(value::Kind::NullValue(_))
        ));

        let missing = semantic_missing("archived_at");
        assert_eq!(missing.operator, SemanticPropertyOperator::Missing as i32);
        assert!(missing.value.is_none());
    }

    #[test]
    fn builder_preserves_conjunctive_filter_inputs() {
        let filter = SemanticEligibilityFilterBuilder::new()
            .require_label("Article")
            .require_node_id("node-1")
            .exists("summary")
            .not_equal("status", string_value("deleted"))
            .build();
        assert_eq!(filter.node_labels, ["Article"]);
        assert_eq!(filter.node_ids, ["node-1"]);
        assert_eq!(filter.properties.len(), 2);
        assert_eq!(
            filter.properties[0].operator,
            SemanticPropertyOperator::Exists as i32
        );
        assert_eq!(
            filter.properties[1].operator,
            SemanticPropertyOperator::NotEqual as i32
        );
    }
}
