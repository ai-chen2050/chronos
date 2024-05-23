use crate::{
    api::{read, write, response::broadcast_srv_state}, 
    zchronod::ZchronodArc
};
use std::{net::SocketAddr, sync::Arc};
use protos::zmessage::ZMessage;
use websocket::ReceiveMessage;
use prost::Message;
use protos::innermsg::{Action, Identity, Innermsg};
use tracing::*;

pub(crate) async fn p2p_event_loop(arc_zchronod: ZchronodArc) {
    info!("Now p2p udp listen on : {}", arc_zchronod.config.inner_p2p);
    loop {
        let mut buf = [0; 65535];
        let (n, src) = arc_zchronod.socket.recv_from(&mut buf).await.unwrap();
        let msg = prost::bytes::Bytes::copy_from_slice(&buf[..n]);
        if let Ok(m) = Innermsg::decode(msg) {
            info!("Received: message from identity: {:?}, action: {:?}", m.identity(), m.action());
            handle_msg(arc_zchronod.clone(), m, src).await;
        } else {
            info!("No action, only support innermsg type between vlc & p2p modules at now");
        }
    }
}

pub(crate) async fn handle_msg(arc_zchronod: ZchronodArc, inner_msg: Innermsg, src: SocketAddr) {
    if let Some(p2p_msg) = &inner_msg.clone().message {
        match inner_msg.identity() {
            Identity::Client => handle_cli_msg(inner_msg, p2p_msg, arc_zchronod, src).await,
            Identity::Server => handle_srv_msg(inner_msg, p2p_msg, arc_zchronod, src).await,
            Identity::Init => {todo!()},
        }
    } else {
        info!("p2p_msg is empty, no action triggered!");
    }
}

async fn handle_srv_msg(inner_msg: Innermsg, p2p_msg: &ZMessage, arc_zchronod: ZchronodArc, src: SocketAddr) {
    let parse_ret = serde_json::from_slice(&p2p_msg.data);
    match parse_ret {
        Err(_) => {
            error!("Err: server_state please to use serde_json serialize");
        }
        Ok(input_state) => {
            let (need_broadcast, merged) = arc_zchronod.state.write().await.merge(&input_state);
            if need_broadcast {
                broadcast_srv_state(arc_zchronod.clone(), inner_msg, src).await;
            }
            if merged {
                let input_clock_info = &input_state.clock_info;
                let state_clock_info = &arc_zchronod.state.read().await.clock_info.clone();
                arc_zchronod.storage.sinker_merge_log(input_clock_info, state_clock_info).await;
            }
        },
    }
}

async fn handle_cli_msg(inner_msg: Innermsg, p2p_msg: &ZMessage, arc_zchronod: ZchronodArc, src: SocketAddr) {
    match inner_msg.action() {
        Action::Write => write::handle_cli_write_msg(arc_zchronod, inner_msg, p2p_msg, src).await,
        Action::Read => read::handle_cli_read_msg(arc_zchronod, inner_msg, p2p_msg, src).await,
        _ => {}
    }
}

/// sample handler
pub(crate) async fn handle_incoming_ws_msg(websocket_url: String) {
    let ws_config = Arc::new(websocket::WebsocketConfig::default());
    let l = websocket::WebsocketListener::bind(ws_config, websocket_url).await.unwrap();

    let _addr = l.local_addr().unwrap();

    let (_send, mut recv) = l.accept().await.unwrap();

    loop {
        let res = recv.recv().await.unwrap();
        match res {
            ReceiveMessage::Request(data, res) => {
                res.respond(data).await.unwrap();
            }
            oth => panic!("unexpected: {oth:?}"),
        }
    }
}