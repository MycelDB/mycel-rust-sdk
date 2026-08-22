use mycel_proto::client::v1::{
    aggregate_argument, expr, value_expr, AggregateArgument, AggregateFunction,
    AggregateProjection, DepthSpec, Expr, GraphPattern, GraphQuery, NodePattern, OrderSpec,
    PropExpr, PropertyEqualsExpr, ReturnProjection, ReturnProjectionKind, SemanticSearchExpr,
    SortDirection, TextSearchExpr, TraversalDirection, TraversalStep, ValueExpr,
};
use prost_types::{value, Value};

pub fn indexed_node_lookup_query(
    alias: impl Into<String>,
    label: impl Into<String>,
    property: impl Into<String>,
    value: Value,
    output_name: impl Into<String>,
) -> GraphQuery {
    let alias = alias.into();
    let output_name = default_output_name(output_name.into(), &alias);
    GraphQuery {
        r#match: Some(GraphPattern {
            start: Some(NodePattern {
                alias: alias.clone(),
                labels: vec![label.into()],
                ..Default::default()
            }),
            ..Default::default()
        }),
        r#where: Some(Expr {
            expr: Some(expr::Expr::PropertyEquals(PropertyEqualsExpr {
                alias: alias.clone(),
                name: property.into(),
                value: Some(value),
            })),
        }),
        returns: vec![node_return(alias, output_name)],
        ..Default::default()
    }
}

pub fn ordered_node_query(
    alias: impl Into<String>,
    label: impl Into<String>,
    property: impl Into<String>,
    direction: SortDirection,
    limit: i32,
) -> GraphQuery {
    let alias = alias.into();
    GraphQuery {
        r#match: Some(GraphPattern {
            start: Some(NodePattern {
                alias: alias.clone(),
                labels: vec![label.into()],
                ..Default::default()
            }),
            ..Default::default()
        }),
        returns: vec![node_return(alias.clone(), alias.clone())],
        order_by: vec![OrderSpec {
            value: Some(prop_value(alias, property)),
            direction: direction as i32,
        }],
        limit,
        ..Default::default()
    }
}

pub fn text_predicate_query(
    alias: impl Into<String>,
    label: impl Into<String>,
    property: impl Into<String>,
    text: impl Into<String>,
    output_name: impl Into<String>,
) -> GraphQuery {
    let alias = alias.into();
    let output_name = default_output_name(output_name.into(), &alias);
    GraphQuery {
        r#match: Some(GraphPattern {
            start: Some(NodePattern {
                alias: alias.clone(),
                labels: vec![label.into()],
                ..Default::default()
            }),
            ..Default::default()
        }),
        r#where: Some(Expr {
            expr: Some(expr::Expr::Text(TextSearchExpr {
                alias: alias.clone(),
                field: property.into(),
                query: text.into(),
            })),
        }),
        returns: vec![node_return(alias, output_name)],
        ..Default::default()
    }
}

pub fn semantic_predicate_query(
    alias: impl Into<String>,
    label: impl Into<String>,
    text: impl Into<String>,
    top_k: i32,
    output_name: impl Into<String>,
) -> GraphQuery {
    let alias = alias.into();
    let output_name = default_output_name(output_name.into(), &alias);
    GraphQuery {
        r#match: Some(GraphPattern {
            start: Some(NodePattern {
                alias: alias.clone(),
                labels: vec![label.into()],
                ..Default::default()
            }),
            ..Default::default()
        }),
        r#where: Some(Expr {
            expr: Some(expr::Expr::Semantic(SemanticSearchExpr {
                alias: alias.clone(),
                field: String::new(),
                query: text.into(),
                rule_ref: String::new(),
                limit: top_k,
                embedding_binding_key: String::new(),
            })),
        }),
        returns: vec![node_return(alias, output_name)],
        ..Default::default()
    }
}

pub fn path_query(
    path_alias: impl Into<String>,
    start_alias: impl Into<String>,
    start_label: impl Into<String>,
    edge_kind: impl Into<String>,
    target_alias: impl Into<String>,
    min_depth: i32,
    max_depth: i32,
) -> GraphQuery {
    let path_alias = path_alias.into();
    GraphQuery {
        r#match: Some(GraphPattern {
            start: Some(NodePattern {
                alias: start_alias.into(),
                labels: vec![start_label.into()],
                ..Default::default()
            }),
            steps: vec![TraversalStep {
                direction: TraversalDirection::Out as i32,
                edge_kind: edge_kind.into(),
                target: Some(NodePattern {
                    alias: target_alias.into(),
                    ..Default::default()
                }),
                depth: Some(DepthSpec {
                    min_depth,
                    max_depth,
                }),
                ..Default::default()
            }],
        }),
        path_alias: path_alias.clone(),
        returns: vec![ReturnProjection {
            alias: path_alias.clone(),
            output_name: path_alias,
            kind: ReturnProjectionKind::Path as i32,
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub fn aggregate_count_query(
    alias: impl Into<String>,
    label: impl Into<String>,
    output_name: impl Into<String>,
) -> GraphQuery {
    let alias = alias.into();
    let output_name = default_output_name(output_name.into(), "count");
    GraphQuery {
        r#match: Some(GraphPattern {
            start: Some(NodePattern {
                alias,
                labels: vec![label.into()],
                ..Default::default()
            }),
            ..Default::default()
        }),
        aggregate_returns: vec![AggregateProjection {
            output_name,
            function: AggregateFunction::Count as i32,
            argument: Some(aggregate_star()),
        }],
        ..Default::default()
    }
}

pub fn aggregate_property_query(
    alias: impl Into<String>,
    label: impl Into<String>,
    property: impl Into<String>,
    function: AggregateFunction,
    output_name: impl Into<String>,
) -> GraphQuery {
    let alias = alias.into();
    GraphQuery {
        r#match: Some(GraphPattern {
            start: Some(NodePattern {
                alias: alias.clone(),
                labels: vec![label.into()],
                ..Default::default()
            }),
            ..Default::default()
        }),
        aggregate_returns: vec![AggregateProjection {
            output_name: output_name.into(),
            function: function as i32,
            argument: Some(aggregate_value(prop_value(alias, property))),
        }],
        ..Default::default()
    }
}

pub fn prop_value(alias: impl Into<String>, property: impl Into<String>) -> ValueExpr {
    ValueExpr {
        expr: Some(value_expr::Expr::Prop(PropExpr {
            alias: alias.into(),
            name: property.into(),
        })),
    }
}

pub fn string_value(value: impl Into<String>) -> Value {
    Value {
        kind: Some(value::Kind::StringValue(value.into())),
    }
}

pub fn aggregate_star() -> AggregateArgument {
    AggregateArgument {
        argument: Some(aggregate_argument::Argument::Star(true)),
    }
}

pub fn aggregate_value(value: ValueExpr) -> AggregateArgument {
    AggregateArgument {
        argument: Some(aggregate_argument::Argument::Value(value)),
    }
}

fn node_return(alias: String, output_name: String) -> ReturnProjection {
    ReturnProjection {
        alias,
        output_name,
        kind: ReturnProjectionKind::Node as i32,
        ..Default::default()
    }
}

fn default_output_name(output_name: String, fallback: &str) -> String {
    if output_name.is_empty() {
        fallback.to_string()
    } else {
        output_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycel_proto::client::v1::{aggregate_argument, expr, value_expr};

    #[test]
    fn builds_common_query_shapes() {
        let lookup = indexed_node_lookup_query("n", "Note", "title", string_value("A"), "node");
        assert_eq!(lookup.r#match.unwrap().start.unwrap().alias, "n");
        match lookup.r#where.unwrap().expr.unwrap() {
            expr::Expr::PropertyEquals(eq) => assert_eq!(eq.name, "title"),
            other => panic!("unexpected where expr: {other:?}"),
        }
        assert_eq!(lookup.returns[0].output_name, "node");

        let ordered = ordered_node_query("j", "JournalEntry", "date", SortDirection::Desc, 10);
        assert_eq!(ordered.limit, 10);
        match ordered.order_by[0].value.clone().unwrap().expr.unwrap() {
            value_expr::Expr::Prop(prop) => assert_eq!(prop.name, "date"),
            other => panic!("unexpected order expr: {other:?}"),
        }

        let text = text_predicate_query("d", "Document", "body", "memory", "doc");
        match text.r#where.unwrap().expr.unwrap() {
            expr::Expr::Text(t) => assert_eq!(t.query, "memory"),
            other => panic!("unexpected text expr: {other:?}"),
        }

        let semantic = semantic_predicate_query("d", "Document", "memory", 5, "doc");
        match semantic.r#where.unwrap().expr.unwrap() {
            expr::Expr::Semantic(s) => assert_eq!(s.limit, 5),
            other => panic!("unexpected semantic expr: {other:?}"),
        }

        let path = path_query("p", "a", "Note", "REFERENCES", "b", 1, 3);
        assert_eq!(path.path_alias, "p");
        assert_eq!(
            path.r#match.unwrap().steps[0]
                .depth
                .as_ref()
                .unwrap()
                .max_depth,
            3
        );

        let count = aggregate_count_query("n", "Note", "total");
        assert_eq!(
            count.aggregate_returns[0].function,
            AggregateFunction::Count as i32
        );
        assert!(matches!(
            count.aggregate_returns[0]
                .argument
                .as_ref()
                .unwrap()
                .argument,
            Some(aggregate_argument::Argument::Star(true))
        ));

        let avg =
            aggregate_property_query("n", "Note", "score", AggregateFunction::Avg, "avg_score");
        assert_eq!(
            avg.aggregate_returns[0].function,
            AggregateFunction::Avg as i32
        );
        match avg.aggregate_returns[0]
            .argument
            .as_ref()
            .unwrap()
            .argument
            .as_ref()
            .unwrap()
        {
            aggregate_argument::Argument::Value(v) => match v.expr.as_ref().unwrap() {
                value_expr::Expr::Prop(prop) => assert_eq!(prop.name, "score"),
                other => panic!("unexpected aggregate value expr: {other:?}"),
            },
            other => panic!("unexpected aggregate arg: {other:?}"),
        }
    }
}
