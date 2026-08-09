use mycel_proto::client::v1::{WatchGraphChangesRequest, WatchGraphChangesResponse};
use tonic::Streaming;

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
    /// Opens the graph-change stream for one space/domain.
    ///
    /// The returned stream may deliver checkpoints, committed graph-change
    /// events, explicit gaps, and heartbeats. Operation IDs in graph-change
    /// origins are correlation metadata only.
    pub async fn watch_graph_changes(
        &mut self,
        request: WatchGraphChangesRequest,
    ) -> Result<Streaming<WatchGraphChangesResponse>> {
        let first = request.clone();
        let res = client_call_with_refresh!(
            self,
            self.graph_change
                .watch_graph_changes(self.auth_request(first)),
            self.graph_change
                .watch_graph_changes(self.auth_request(request))
        )?;
        Ok(res.into_inner())
    }
}
