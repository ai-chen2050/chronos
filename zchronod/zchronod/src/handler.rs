use crate::{
    api::{read, write}, 
    zchronod::ZchronodArc,
};
use std::{net::SocketAddr, sync::Arc};
use protos::{vlc::{ClockType, ZClock}, zmessage::{ZMessage, ZType}};
use websocket::ReceiveMessage;
use prost::Message;
use protos::innermsg::{Action, Identity, Innermsg};
use tracing::*;

pub(crate) async fn p2p_event_loop(arc_zchronod: ZchronodArc) {
    info!("Now p2p udp listen on : {}", arc_zchronod.config.net.inner_p2p);
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
    match p2p_msg.r#type() {
        ZType::Clock => {
            let clock_msg = prost::bytes::Bytes::from(p2p_msg.data.clone());
            let z_clock = ZClock::decode(clock_msg).unwrap_or(ZClock::default());
            match z_clock.r#type() {
                ClockType::EventTrigger => write::handle_srv_event_trigger(z_clock, inner_msg, p2p_msg, arc_zchronod, src).await,
                ClockType::DiffReq => todo!(),
                ClockType::DiffRsp => todo!(),
                ClockType::ActiveSync => todo!(),
            }
        }
        _ => error!("Server message: just support ZType::Clock for state sync & clock update!"),
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