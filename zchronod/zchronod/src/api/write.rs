use std::collections::BTreeSet;
use std::net::SocketAddr;
use crate::vlc::ClockInfo;
use protos::innermsg::Innermsg;
use protos::zmessage::{ZMessage, ZType};
use protos::bussiness::ZChat;
use prost::Message;
use crate::zchronod::ZchronodArc;
use tracing::*;

use super::response::broadcast_srv_state;

pub async fn handle_cli_write_msg(arc_zchronod: ZchronodArc, inner_msg: Innermsg, p2p_msg: &ZMessage, src: SocketAddr) {
    match p2p_msg.r#type() {
        ZType::Zchat =>{
            let zchat_msg = prost::bytes::Bytes::from(p2p_msg.data.clone());
            let m = ZChat::decode(zchat_msg).unwrap();
            if arc_zchronod.state.write().await.add(BTreeSet::from_iter(vec![m.message_data.clone()])) {
                let update_clock_info: ClockInfo = arc_zchronod.state.read().await.clock_info.clone();
                let state_storage = &arc_zchronod.clone().storage;
                state_storage.sinker_clock(hex::encode(p2p_msg.id.clone()),
                                        m.message_data, &update_clock_info).await;
                state_storage.sinker_zmessage(p2p_msg.clone()).await;
                broadcast_srv_state(arc_zchronod, inner_msg, src).await;
            }
        }
        _ => info!("Write: now just support ZType::Zchat = 4!"),
    }
}