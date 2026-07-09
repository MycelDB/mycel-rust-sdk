use mycel_proto::client::v1::{
    ApplyGraphOperationsRequest, ApplyGraphOperationsResponse, CreateEdgeRequest,
    CreateNodeRequest, DeleteNodeRequest, Edge, EdgeCreate, ExecuteQueryRequest,
    ExecuteQueryResponse, GetNodeRequest, GetParentRequest, GetParentResponse, GraphOperation,
    GraphQuery, ListChildrenRequest, ListChildrenResponse, ListNodesRequest, ListNodesResponse,
    Node, NodeCreate, UpdateNodeRequest,
};
use prost_types::FieldMask;

use crate::{
    auth::is_expired_unauthenticated,
    client::Client,
    error::{Error, Result},
};

macro_rules! client_call_with_refresh {
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

impl Client {
    pub async fn create_node(
        &mut self,
        transaction_id: impl Into<String>,
        node: NodeCreate,
    ) -> Result<Node> {
        let transaction_id = transaction_id.into();
        let res = client_call_with_refresh!(
            self,
            self.graph.create_node(self.auth_request(CreateNodeRequest {
                transaction_id: transaction_id.clone(),
                node: Some(node.clone()),
            })),
            self.graph.create_node(self.auth_request(CreateNodeRequest {
                transaction_id,
                node: Some(node),
            }))
        )?
        .into_inner();
        res.node
            .ok_or_else(|| Error::Message("create node response did not include a node".into()))
    }

    pub async fn apply_graph_operations(
        &mut self,
        transaction_id: impl Into<String>,
        operations: Vec<GraphOperation>,
    ) -> Result<ApplyGraphOperationsResponse> {
        let transaction_id = transaction_id.into();
        let res = client_call_with_refresh!(
            self,
            self.graph
                .apply_graph_operations(self.auth_request(ApplyGraphOperationsRequest {
                    transaction_id: transaction_id.clone(),
                    operations: operations.clone(),
                })),
            self.graph
                .apply_graph_operations(self.auth_request(ApplyGraphOperationsRequest {
                    transaction_id,
                    operations,
                }))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn update_node_content(
        &mut self,
        transaction_id: impl Into<String>,
        node_id: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<Node> {
        let transaction_id = transaction_id.into();
        let node_id = node_id.into();
        let content = content.into();
        let req = |transaction_id: String, node_id: String, content: String| UpdateNodeRequest {
            transaction_id,
            node: Some(Node {
                node_id,
                content,
                ..Default::default()
            }),
            update_mask: Some(FieldMask {
                paths: vec!["content".to_string()],
            }),
        };
        let res = client_call_with_refresh!(
            self,
            self.graph.update_node(self.auth_request(req(
                transaction_id.clone(),
                node_id.clone(),
                content.clone()
            ))),
            self.graph
                .update_node(self.auth_request(req(transaction_id, node_id, content)))
        )?
        .into_inner();
        res.node
            .ok_or_else(|| Error::Message("update node response did not include a node".into()))
    }

    pub async fn delete_node(
        &mut self,
        transaction_id: impl Into<String>,
        node_id: impl Into<String>,
        recursive: bool,
    ) -> Result<()> {
        let transaction_id = transaction_id.into();
        let node_id = node_id.into();
        client_call_with_refresh!(
            self,
            self.graph.delete_node(self.auth_request(DeleteNodeRequest {
                transaction_id: transaction_id.clone(),
                node_id: node_id.clone(),
                recursive,
            })),
            self.graph.delete_node(self.auth_request(DeleteNodeRequest {
                transaction_id,
                node_id,
                recursive,
            }))
        )?;
        Ok(())
    }

    pub async fn create_edge(
        &mut self,
        transaction_id: impl Into<String>,
        edge: EdgeCreate,
    ) -> Result<Edge> {
        let transaction_id = transaction_id.into();
        let res = client_call_with_refresh!(
            self,
            self.graph.create_edge(self.auth_request(CreateEdgeRequest {
                transaction_id: transaction_id.clone(),
                edge: Some(edge.clone()),
            })),
            self.graph.create_edge(self.auth_request(CreateEdgeRequest {
                transaction_id,
                edge: Some(edge),
            }))
        )?
        .into_inner();
        res.edge
            .ok_or_else(|| Error::Message("create edge response did not include an edge".into()))
    }

    pub async fn get_node(
        &mut self,
        transaction_id: impl Into<String>,
        node_id: impl Into<String>,
    ) -> Result<Node> {
        let transaction_id = transaction_id.into();
        let node_id = node_id.into();
        let res = client_call_with_refresh!(
            self,
            self.graph.get_node(self.auth_request(GetNodeRequest {
                transaction_id: transaction_id.clone(),
                node_id: node_id.clone(),
            })),
            self.graph.get_node(self.auth_request(GetNodeRequest {
                transaction_id,
                node_id,
            }))
        )?
        .into_inner();
        res.node
            .ok_or_else(|| Error::Message("get node response did not include a node".into()))
    }

    pub async fn list_nodes(
        &mut self,
        transaction_id: impl Into<String>,
        page_size: i32,
        page_token: impl Into<String>,
    ) -> Result<ListNodesResponse> {
        let transaction_id = transaction_id.into();
        let page_token = page_token.into();
        let res = client_call_with_refresh!(
            self,
            self.graph.list_nodes(self.auth_request(ListNodesRequest {
                transaction_id: transaction_id.clone(),
                page_size,
                page_token: page_token.clone(),
            })),
            self.graph.list_nodes(self.auth_request(ListNodesRequest {
                transaction_id,
                page_size,
                page_token,
            }))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn list_children(
        &mut self,
        transaction_id: impl Into<String>,
        parent_node_id: impl Into<String>,
    ) -> Result<ListChildrenResponse> {
        let transaction_id = transaction_id.into();
        let parent_node_id = parent_node_id.into();
        let res = client_call_with_refresh!(
            self,
            self.graph
                .list_children(self.auth_request(ListChildrenRequest {
                    transaction_id: transaction_id.clone(),
                    parent_node_id: parent_node_id.clone(),
                })),
            self.graph
                .list_children(self.auth_request(ListChildrenRequest {
                    transaction_id,
                    parent_node_id,
                }))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn get_parent(
        &mut self,
        transaction_id: impl Into<String>,
        child_node_id: impl Into<String>,
    ) -> Result<GetParentResponse> {
        let transaction_id = transaction_id.into();
        let child_node_id = child_node_id.into();
        let res = client_call_with_refresh!(
            self,
            self.graph.get_parent(self.auth_request(GetParentRequest {
                transaction_id: transaction_id.clone(),
                child_node_id: child_node_id.clone(),
            })),
            self.graph.get_parent(self.auth_request(GetParentRequest {
                transaction_id,
                child_node_id,
            }))
        )?
        .into_inner();
        Ok(res)
    }

    pub async fn execute_query(
        &mut self,
        transaction_id: impl Into<String>,
        query: GraphQuery,
        page_size: i32,
    ) -> Result<ExecuteQueryResponse> {
        let transaction_id = transaction_id.into();
        let res = client_call_with_refresh!(
            self,
            self.query
                .execute_query(self.auth_request(ExecuteQueryRequest {
                    transaction_id: transaction_id.clone(),
                    query: Some(query.clone()),
                    page_size,
                    page_token: String::new(),
                })),
            self.query
                .execute_query(self.auth_request(ExecuteQueryRequest {
                    transaction_id,
                    query: Some(query),
                    page_size,
                    page_token: String::new(),
                }))
        )?
        .into_inner();
        Ok(res)
    }
}
