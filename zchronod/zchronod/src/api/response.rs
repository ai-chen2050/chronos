use crate::zchronod::ZchronodArc;
use prost::Message;
use protos::{
    bussiness::QueryResponse,
    innermsg::{Action, Identity, Innermsg, PushType},
};
use std::net::SocketAddr;
use tracing::*;

pub fn make_query_response(success: bool, reason: String, data: &[u8]) -> QueryResponse {
    let response = QueryResponse {
        success,
        reason,
        data: data.to_vec(),
    };

    response
}

pub async fn broadcast_srv_state(arc_zchronod: ZchronodArc, mut inner: Innermsg, src: SocketAddr) {
    let state_value = arc_zchronod.as_ref().state.read().await;
    let serde_res = serde_json::to_string(&*state_value);
    let serde_string = &serde_res.unwrap();
    let state_data = serde_string.as_bytes();

    let mut p2p_msg = inner.message.unwrap();
    p2p_msg.data = state_data.to_vec();
    inner.message = Some(p2p_msg);
    inner.identity = Identity::Server.into();
    inner.action = Action::WriteReply.into();
    inner.push_type = PushType::Broadcast.into();

    let mut buf = vec![];
    inner.encode(&mut buf).unwrap();
    info!("Response Srv: {:?}", inner);
    arc_zchronod.socket.send_to(&buf, src).await.unwrap_or(0);
}

pub(crate) async fn respond_cli_query(arc_zchronod: ZchronodArc, mut inner: Innermsg, p2p_data: &[u8], src: SocketAddr) {
    let mut p2p_msg = inner.message.unwrap();
    p2p_msg.data = p2p_data.to_vec();
    inner.message = Some(p2p_msg);
    inner.identity = Identity::Server.into();
    inner.action = Action::ReadReply.into();
    inner.push_type = PushType::Direct.into();

    let mut buf = vec![];
    inner.encode(&mut buf).unwrap();
    info!("Response Cli: {:?}", inner);
    arc_zchronod.socket.send_to(&buf, src).await.unwrap();
}
