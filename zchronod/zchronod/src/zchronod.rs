use std::io::Read;
use std::sync::RwLock;
use std::{cmp, collections::BTreeSet, sync::Arc};
use node_api::config::ZchronodConfig;
use prost::Message;
use sea_orm::Identity;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use websocket::ReceiveMessage;
use crate::{node_factory::ZchronodFactory, storage::Storage, vlc::Clock};
use serde::{Deserialize, Serialize};
use protos::zmessage::{ZAction, ZIdentity, ZMessage, ZType};
use protos::bussiness::{ZChat};

pub struct Zchronod {
    pub config: ZchronodConfig,
    pub socket: UdpSocket,
    pub storage: Storage,
    pub state: ServerState,
}

pub type ZchronodArc = Arc<RwLock<Zchronod>>;

impl Zchronod {
    pub fn zchronod_factory() -> ZchronodFactory {
        ZchronodFactory::init()
    }

    async fn handle_msg(&mut self, msg: String) {
        
    }
}

/// Clock info sinker to db.
/// id is server node id, count is the event count in this server.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClockInfo {
    pub clock: Clock,
    pub node_id: String,  
    pub message_id: String,
    pub count: u128,
    pub create_at: u128,
}

impl ClockInfo {
    fn new(clock: Clock, node_id: String, message_id: String, count: u128) -> Self {
        let create_at = tools::helper::get_time_ms();
        Self { clock, node_id, message_id, count, create_at }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerState {
    clock_info: ClockInfo,
    id: String,
    items: BTreeSet<String>,
}

impl ServerState {
    /// Create a new server state.
    pub fn new(node_id: String) -> Self {
        Self {
            clock_info: ClockInfo::new(Clock::new(), node_id.clone(), "".to_owned(), 0),
            id: node_id,
            items: BTreeSet::new(),
        }
    }

    /// Add items into the state. Returns true if resulting in a new state.
    fn add(&mut self, items: BTreeSet<String>) -> bool {
        if items.is_subset(&self.items) {
            false
        } else {
            self.items.extend(items);
            self.clock_info.clock.inc(self.id.clone());
            true
        }
    }

    /// Merge another ServerState into the current state. Returns true if
    /// resulting in a new state (different from current and received
    /// state).
    fn merge(&mut self, other: &Self) -> bool {
        match self.clock_info.clock.partial_cmp(&other.clock_info.clock) {
            Some(cmp::Ordering::Equal) => false,
            Some(cmp::Ordering::Greater) => false,
            Some(cmp::Ordering::Less) => {
                self.clock_info.clock = other.clock_info.clock.clone();
                self.items = other.items.clone();
                false
            }
            None => {
                self.clock_info.clock.merge(&vec![&other.clock_info.clock]);
                self.add(other.items.clone())
            }
        }
    }
}

pub(crate) async fn p2p_event_loop(arc_zchronod: Arc<Mutex<Zchronod>>) {
    loop {
        let mut buf = [0; 1500];
        let (n, _) = arc_zchronod.lock().await.socket.recv_from(&mut buf).await.unwrap();
        let msg = prost::bytes::Bytes::copy_from_slice(&buf[..n]);
        let m = ZMessage::decode(msg).unwrap();
        // arc_zchronod.handle_msg(msg).await;
        handle_msg(arc_zchronod.clone(), m).await;
    }
}

pub(crate) async fn handle_msg(arc_zchronod: Arc<Mutex<Zchronod>>, msg: ZMessage) {
    match msg.identity() {
        ZIdentity::UTypeCli => {
            match msg.action() {
                ZAction::ZTypeWrite => {
                    match msg.r#type() {
                        ZType::Zchat =>{
                            let zchat_msg = prost::bytes::Bytes::from(msg.clone().data);
                            let m = ZChat::decode(zchat_msg).unwrap();
                            if arc_zchronod.lock().await.state.add(BTreeSet::from_iter(vec![m.message_data])) {
                                broadcast_state(arc_zchronod, msg).await;
                            }
                        }
                        _ => println!("todo!"),
                    }
                }
                ZAction::ZTypeRead => {
                    todo!()
                }
            }
        }
        ZIdentity::UTypeSer => {
            let input_state: ServerState = serde_json::from_slice(&msg.data).unwrap();
            if arc_zchronod.lock().await.state.merge(&input_state) {
                // self.broadcast_state().await;
                broadcast_state(arc_zchronod, msg).await;
            }
        }
    }
}

pub(crate) async fn broadcast_state(arc_zchronod: Arc<Mutex<Zchronod>>, msg: ZMessage) {
    let serde_res = serde_json::to_string(&arc_zchronod.lock().await.state);
    let serde_string = &serde_res.unwrap();
    let state_data = serde_string.as_bytes();
    let msg = ZMessage {
        id: arc_zchronod.lock().await.state.id.clone().into_bytes(),
        from: Vec::from(msg.from.clone()),
        to: Vec::from(msg.to.clone()),
        data: state_data.to_vec(),
        identity: 1,
        ..Default::default()
    };
    let mut buf2 = vec![];
    msg.encode(&mut buf2).unwrap();
    println!("buf: {:?}", buf2);
    arc_zchronod.lock().await.socket.send_to(&buf2, &arc_zchronod.lock().await.config.outer_p2p).await.unwrap();
}
/// sample handler
pub(crate) async fn handle_incoming_ws_msg() {
    let ws_config = Arc::new(websocket::WebsocketConfig::default());
    let l = websocket::WebsocketListener::bind(ws_config, "127.0.0.1:8080").await.unwrap();

    let addr = l.local_addr().unwrap();

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