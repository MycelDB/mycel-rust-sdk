use mycel_proto::client::v1::{
    ApplyGraphOperationsRequest, ApplyGraphOperationsResponse, CreateEdgeRequest,
    CreateNodeRequest, DeleteNodeRequest, Edge, EdgeCreate, ExecuteQueryRequest,
    ExecuteQueryResponse, GetNodeRequest, GetParentRequest, GetParentResponse, GraphOperation,
    GraphQuery, ListChildrenRequest, ListChildrenResponse, ListNodesRequest, ListNodesResponse,
    Node, NodeCreate, UpdateNodeRequest,
};
use prost_types::FieldMask;

use crate::{
    client::Client,
    error::{Error, Result},
};

impl Client {
    pub async fn create_node(
        &mut self,
        transaction_id: impl Into<String>,
        node: NodeCreate,
    ) -> Result<Node> {
        let res = self
            .graph
            .create_node(self.auth_request(CreateNodeRequest {
                transaction_id: transaction_id.into(),
                node: Some(node),
            }))
            .await?
            .into_inner();
        res.node
            .ok_or_else(|| Error::Message("create node response did not include a node".into()))
    }

    pub async fn apply_graph_operations(
        &mut self,
        transaction_id: impl Into<String>,
        operations: Vec<GraphOperation>,
    ) -> Result<ApplyGraphOperationsResponse> {
        Ok(self
            .graph
            .apply_graph_operations(self.auth_request(ApplyGraphOperationsRequest {
                transaction_id: transaction_id.into(),
                operations,
            }))
            .await?
            .into_inner())
    }

    pub async fn update_node_content(
        &mut self,
        transaction_id: impl Into<String>,
        node_id: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<Node> {
        let res = self
            .graph
            .update_node(self.auth_request(UpdateNodeRequest {
                transaction_id: transaction_id.into(),
                node: Some(Node {
                    node_id: node_id.into(),
                    content: content.into(),
                    ..Default::default()
                }),
                update_mask: Some(FieldMask {
                    paths: vec!["content".to_string()],
                }),
            }))
            .await?
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
        self.graph
            .delete_node(self.auth_request(DeleteNodeRequest {
                transaction_id: transaction_id.into(),
                node_id: node_id.into(),
                recursive,
            }))
            .await?;
        Ok(())
    }

    pub async fn create_edge(
        &mut self,
        transaction_id: impl Into<String>,
        edge: EdgeCreate,
    ) -> Result<Edge> {
        let res = self
            .graph
            .create_edge(self.auth_request(CreateEdgeRequest {
                transaction_id: transaction_id.into(),
                edge: Some(edge),
            }))
            .await?
            .into_inner();
        res.edge
            .ok_or_else(|| Error::Message("create edge response did not include an edge".into()))
    }

    pub async fn get_node(
        &mut self,
        transaction_id: impl Into<String>,
        node_id: impl Into<String>,
    ) -> Result<Node> {
        let res = self
            .graph
            .get_node(self.auth_request(GetNodeRequest {
                transaction_id: transaction_id.into(),
                node_id: node_id.into(),
            }))
            .await?
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
        Ok(self
            .graph
            .list_nodes(self.auth_request(ListNodesRequest {
                transaction_id: transaction_id.into(),
                page_size,
                page_token: page_token.into(),
            }))
            .await?
            .into_inner())
    }

    pub async fn list_children(
        &mut self,
        transaction_id: impl Into<String>,
        parent_node_id: impl Into<String>,
    ) -> Result<ListChildrenResponse> {
        Ok(self
            .graph
            .list_children(self.auth_request(ListChildrenRequest {
                transaction_id: transaction_id.into(),
                parent_node_id: parent_node_id.into(),
            }))
            .await?
            .into_inner())
    }

    pub async fn get_parent(
        &mut self,
        transaction_id: impl Into<String>,
        child_node_id: impl Into<String>,
    ) -> Result<GetParentResponse> {
        Ok(self
            .graph
            .get_parent(self.auth_request(GetParentRequest {
                transaction_id: transaction_id.into(),
                child_node_id: child_node_id.into(),
            }))
            .await?
            .into_inner())
    }

    pub async fn execute_query(
        &mut self,
        transaction_id: impl Into<String>,
        query: GraphQuery,
        page_size: i32,
    ) -> Result<ExecuteQueryResponse> {
        Ok(self
            .query
            .execute_query(self.auth_request(ExecuteQueryRequest {
                transaction_id: transaction_id.into(),
                query: Some(query),
                page_size,
                page_token: String::new(),
            }))
            .await?
            .into_inner())
    }
}
