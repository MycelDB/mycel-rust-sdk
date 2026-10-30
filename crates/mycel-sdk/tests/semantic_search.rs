use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::SystemTime,
};

use mycel::{
    client::v1::{
        SemanticResultProvenance, SemanticSearchRequest, SemanticSearchResponse,
        SemanticSearchResult,
    },
    common::v1::{RefreshRequest, RefreshResponse},
};
use mycel_sdk::{
    semantic::{string_value, SemanticEligibilityFilterBuilder},
    Config, Error,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    body::BoxBody,
    codegen::{http, Body, BoxFuture, Service, StdError},
    server::{NamedService, UnaryService},
    transport::Server,
    Code, Request, Response, Status,
};

#[derive(Clone, Copy)]
enum SemanticBehavior {
    Success,
    ExpiredThenSuccess,
    FailedPrecondition,
}

struct ScriptState {
    behavior: SemanticBehavior,
    semantic_requests: Vec<SemanticSearchRequest>,
    semantic_authorization: Vec<String>,
    refresh_requests: Vec<RefreshRequest>,
    response: SemanticSearchResponse,
}

impl ScriptState {
    fn new(behavior: SemanticBehavior) -> Self {
        Self {
            behavior,
            semantic_requests: Vec::new(),
            semantic_authorization: Vec::new(),
            refresh_requests: Vec::new(),
            response: populated_response(),
        }
    }
}

#[tokio::test]
async fn semantic_search_forwards_exact_request_after_proactive_refresh() {
    let state = Arc::new(Mutex::new(ScriptState::new(SemanticBehavior::Success)));
    let (addr, server) = spawn_server(Arc::clone(&state)).await;
    let mut client = mycel_sdk::dial(Config {
        addr,
        access_token: "nearly-expired".into(),
        access_token_expire_time: Some(SystemTime::UNIX_EPOCH),
        refresh_token: "refresh-token".into(),
        ..Default::default()
    })
    .await
    .unwrap();
    let request = search_request();

    let response = client.semantic_search(request.clone()).await.unwrap();

    let state = state.lock().unwrap();
    assert_eq!(state.refresh_requests.len(), 1);
    assert_eq!(state.semantic_requests, [request]);
    assert_eq!(state.semantic_authorization, ["Bearer refreshed-access"]);
    assert_eq!(response, state.response);
    assert_eq!(response.next_page_token, "opaque-next");
    assert_eq!(response.filter_execution_mode, "semantic_index_pushdown");
    assert_eq!(response.examined_semantic_candidates, 12);
    assert_eq!(response.eligible_semantic_candidates, 4);
    assert_eq!(response.warnings, ["index warming"]);
    assert_eq!(
        response.results[0]
            .provenance
            .as_ref()
            .unwrap()
            .semantic_generation,
        17
    );
    server.abort();
}

#[tokio::test]
async fn semantic_search_expired_token_retry_replays_exact_request_once() {
    let state = Arc::new(Mutex::new(ScriptState::new(
        SemanticBehavior::ExpiredThenSuccess,
    )));
    let (addr, server) = spawn_server(Arc::clone(&state)).await;
    let mut client = mycel_sdk::dial(Config {
        addr,
        access_token: "expired-access".into(),
        refresh_token: "refresh-token".into(),
        ..Default::default()
    })
    .await
    .unwrap();
    let request = search_request();

    let response = client.semantic_search(request.clone()).await.unwrap();

    let state = state.lock().unwrap();
    assert_eq!(state.refresh_requests.len(), 1);
    assert_eq!(state.semantic_requests, [request.clone(), request]);
    assert_eq!(
        state.semantic_authorization,
        ["Bearer expired-access", "Bearer refreshed-access"]
    );
    assert_eq!(response, state.response);
    server.abort();
}

#[tokio::test]
async fn semantic_search_failed_precondition_is_not_retried_or_restarted() {
    let state = Arc::new(Mutex::new(ScriptState::new(
        SemanticBehavior::FailedPrecondition,
    )));
    let (addr, server) = spawn_server(Arc::clone(&state)).await;
    let mut client = mycel_sdk::dial(Config {
        addr,
        access_token: "valid-access".into(),
        refresh_token: "refresh-token".into(),
        ..Default::default()
    })
    .await
    .unwrap();
    let request = search_request();

    let error = client.semantic_search(request.clone()).await.unwrap_err();

    assert!(matches!(
        error,
        Error::Status(status)
            if status.code() == Code::FailedPrecondition
                && status.message() == "stale semantic continuation token"
    ));
    let state = state.lock().unwrap();
    assert!(state.refresh_requests.is_empty());
    assert_eq!(state.semantic_requests, [request]);
    server.abort();
}

fn search_request() -> SemanticSearchRequest {
    SemanticSearchRequest {
        space_id: "space-1".into(),
        domain_id: "domain-1".into(),
        semantic_rule_id: Some("rule-1".into()),
        embedding_binding_key: Some("primary".into()),
        query: "raft recovery".into(),
        limit: 25,
        min_score: Some(0.75),
        page_token: Some("opaque-current".into()),
        filter: Some(
            SemanticEligibilityFilterBuilder::new()
                .require_label("Article")
                .require_node_id("node-1")
                .equal("status", string_value("published"))
                .missing("deleted_at")
                .build(),
        ),
    }
}

fn populated_response() -> SemanticSearchResponse {
    SemanticSearchResponse {
        results: vec![SemanticSearchResult {
            semantic_rule_id: "rule-1".into(),
            embedding_binding_key: "primary".into(),
            record_id: "record-1".into(),
            node_id: "node-1".into(),
            score: 0.91,
            matched_chunk_ids: vec!["chunk-1".into()],
            snippet: "raft recovery".into(),
            provenance: Some(SemanticResultProvenance {
                source_graph_revision: 23,
                semantic_generation: 17,
                semantic_configuration_hash: "config-hash".into(),
                opaque_source_version: "source-version".into(),
                dimensions: 3,
                vector_space_key: "primary".into(),
                configured_intelligence_profile_id: "profile-1".into(),
                configured_model_endpoint_id: "endpoint-1".into(),
                configured_model_id: "model-1".into(),
                configured_capability_id: "capability-1".into(),
                actual_route_known: true,
                actual_model_endpoint_id: "endpoint-1".into(),
                actual_model_id: "model-1".into(),
                actual_capability_id: "capability-1".into(),
                rebuild_epoch: "epoch-1".into(),
                generated_at: "2026-01-01T00:00:00Z".into(),
            }),
            ..Default::default()
        }],
        warnings: vec!["index warming".into()],
        next_page_token: "opaque-next".into(),
        filter_execution_mode: "semantic_index_pushdown".into(),
        examined_semantic_candidates: 12,
        eligible_semantic_candidates: 4,
    }
}

async fn spawn_server(state: Arc<Mutex<ScriptState>>) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let incoming = TcpListenerStream::new(listener);
    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(AuthGrpcService {
                state: Arc::clone(&state),
            })
            .add_service(SemanticGrpcService { state })
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });
    (addr.to_string(), server)
}

#[derive(Clone)]
struct SemanticGrpcService {
    state: Arc<Mutex<ScriptState>>,
}

impl NamedService for SemanticGrpcService {
    const NAME: &'static str = "mycel.client.v1.SemanticService";
}

impl<B> Service<http::Request<B>> for SemanticGrpcService
where
    B: Body + Send + 'static,
    B::Error: Into<StdError> + Send + 'static,
{
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: http::Request<B>) -> Self::Future {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            match request.uri().path() {
                "/mycel.client.v1.SemanticService/SemanticSearch" => {
                    let mut grpc = tonic::server::Grpc::new(tonic::codec::ProstCodec::default());
                    Ok(grpc.unary(SemanticSearchRpc { state }, request).await)
                }
                _ => Ok(unimplemented_response()),
            }
        })
    }
}

struct SemanticSearchRpc {
    state: Arc<Mutex<ScriptState>>,
}

impl UnaryService<SemanticSearchRequest> for SemanticSearchRpc {
    type Response = SemanticSearchResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;

    fn call(&mut self, request: Request<SemanticSearchRequest>) -> Self::Future {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            let authorization = request
                .metadata()
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_string();
            let mut state = state.lock().unwrap();
            state.semantic_authorization.push(authorization);
            state.semantic_requests.push(request.into_inner());
            match state.behavior {
                SemanticBehavior::ExpiredThenSuccess if state.semantic_requests.len() == 1 => {
                    Err(Status::unauthenticated("authorization token is expired"))
                }
                SemanticBehavior::FailedPrecondition => Err(Status::failed_precondition(
                    "stale semantic continuation token",
                )),
                _ => Ok(Response::new(state.response.clone())),
            }
        })
    }
}

#[derive(Clone)]
struct AuthGrpcService {
    state: Arc<Mutex<ScriptState>>,
}

impl NamedService for AuthGrpcService {
    const NAME: &'static str = "mycel.common.v1.AuthService";
}

impl<B> Service<http::Request<B>> for AuthGrpcService
where
    B: Body + Send + 'static,
    B::Error: Into<StdError> + Send + 'static,
{
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: http::Request<B>) -> Self::Future {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            match request.uri().path() {
                "/mycel.common.v1.AuthService/Refresh" => {
                    let mut grpc = tonic::server::Grpc::new(tonic::codec::ProstCodec::default());
                    Ok(grpc.unary(RefreshRpc { state }, request).await)
                }
                _ => Ok(unimplemented_response()),
            }
        })
    }
}

struct RefreshRpc {
    state: Arc<Mutex<ScriptState>>,
}

impl UnaryService<RefreshRequest> for RefreshRpc {
    type Response = RefreshResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;

    fn call(&mut self, request: Request<RefreshRequest>) -> Self::Future {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            state
                .lock()
                .unwrap()
                .refresh_requests
                .push(request.into_inner());
            Ok(Response::new(RefreshResponse {
                access_token: "refreshed-access".into(),
                refresh_token: Some("rotated-refresh".into()),
                auth_session_id: "session-1".into(),
                ..Default::default()
            }))
        })
    }
}

fn unimplemented_response() -> http::Response<BoxBody> {
    http::Response::builder()
        .status(200)
        .header("grpc-status", Code::Unimplemented as i32)
        .header("content-type", "application/grpc")
        .body(tonic::body::empty_body())
        .unwrap()
}
