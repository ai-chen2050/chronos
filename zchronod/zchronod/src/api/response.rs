use crate::zchronod::ZchronodArc;
use prost::Message;
use protos::{
    bussiness::QueryResponse,
    innermsg::{Action, Identity, Innermsg, PushType},
};
use crate::vlc::ClockInfo;
use protos::vlc::Clock as ProtoClock;
use protos::vlc::ClockInfo as ProtoClockInfo;
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

pub fn clockinfo_to_proto() -> impl FnMut(ClockInfo) -> ProtoClockInfo {
    move |clock_info| {
        let node_id = hex::decode(clock_info.node_id).unwrap_or_else(|_| Vec::new());
        let clock_hash = hex::decode(clock_info.clock_hash).unwrap_or_else(|_| Vec::new());
        let msg_id = hex::decode(clock_info.message_id).unwrap_or_else(|_| Vec::new());
        ProtoClockInfo {
            clock: Some(ProtoClock {
                values: clock_info.clock.values.into_iter().map(|(k, v)| (k, v as u64)).collect(),
            }),
            node_id,
            clock_hash,
            message_id: msg_id,
            count: clock_info.count as u64,
            create_at: clock_info.create_at as u64,
        }
    }
}

pub async fn broadcast_srv_state(arc_zchronod: ZchronodArc, mut inner: Innermsg, p2p_data: &[u8], src: SocketAddr) {
    let mut p2p_msg = inner.message.unwrap();
    p2p_msg.data = p2p_data.to_vec();
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
