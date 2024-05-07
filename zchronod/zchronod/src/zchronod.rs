use std::io::Read;
use std::net::SocketAddr;
use std::sync::RwLock;
use std::{cmp, collections::BTreeSet, sync::Arc};
use node_api::config::ZchronodConfig;
use prost::Message;
use protos::vlc::ClockInfo as PClockInfo;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, Mutex};
use websocket::ReceiveMessage;
use crate::{node_factory::ZchronodFactory, storage::Storage, vlc::Clock};
use serde::{Deserialize, Serialize};
use protos::zmessage::{ZAction, ZIdentity, ZMessage, ZType};
use protos::bussiness::ZChat;

pub struct Zchronod {
    pub config: ZchronodConfig,
    pub socket: UdpSocket,
    pub storage: Storage,
    pub state: ServerState,
}

pub type ZchronodArc = Arc<Mutex<Zchronod>>;

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

impl From<&PClockInfo> for ClockInfo {
    fn from(protobuf_clock_info: &PClockInfo) -> Self {
        let clock = protobuf_clock_info
            .clock
            .as_ref()
            .map(|c| {
                Clock {
                    values: c
                        .values
                        .iter()
                        .map(|(k, v)| (k.clone(), *v as u128))
                        .collect(),
                }
            }).unwrap();

        let node_id = String::from_utf8_lossy(&protobuf_clock_info.id).into_owned();
        let message_id = String::from_utf8_lossy(&protobuf_clock_info.message_id).into_owned();
        let count = protobuf_clock_info.count;
        let create_at = protobuf_clock_info.create_at;

        ClockInfo {
            clock,
            node_id,
            message_id,
            count: count.into(),
            create_at: create_at.into(),
        }
    }
}

/// MergeLog sinker to db.
/// id is server node id, count is the event count in this server.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MergeLog {
    from_id: String,
    to_id: String,    // to_node trigger merge action
    start_count: u128,
    end_count: u128,
    s_clock_hash: String,    // todo: needs hash when use related-db
    e_clock_hash: String,
    merge_at: u128,
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
    pub fn add(&mut self, items: BTreeSet<String>) -> bool {
        if items.is_subset(&self.items) {
            println!("duplicate message, no action");
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
    pub fn merge(&mut self, other: &Self) -> (bool, bool) {
        match self.clock_info.clock.partial_cmp(&other.clock_info.clock) {
            Some(cmp::Ordering::Equal) => (false, false),
            Some(cmp::Ordering::Greater) => (false, false),
            Some(cmp::Ordering::Less) => {
                self.clock_info.clock = other.clock_info.clock.clone();
                self.items = other.items.clone();
                (false, true)
            }
            None => {
                self.clock_info.clock.merge(&vec![&other.clock_info.clock]);
                let added = self.add(other.items.clone());
                (added, added)
            }
        }
    }
}

pub(crate) async fn p2p_event_loop(arc_zchronod: Arc<Mutex<Zchronod>>) {
    println!("Now p2p udp listen on : {}", arc_zchronod.lock().await.config.inner_p2p);
    loop {
        let mut buf = [0; 1500];
        let (n, src) = arc_zchronod.lock().await.socket.recv_from(&mut buf).await.unwrap();
        let msg = prost::bytes::Bytes::copy_from_slice(&buf[..n]);
        let m = ZMessage::decode(msg).unwrap();
        // arc_zchronod.handle_msg(msg).await;
        handle_msg(arc_zchronod.clone(), m, src).await;
    }
}

pub(crate) async fn handle_msg(arc_zchronod: Arc<Mutex<Zchronod>>, msg: ZMessage, src: SocketAddr) {
    match msg.identity() {
        ZIdentity::UTypeCli => {
            match msg.action() {
                ZAction::ZTypeWrite => {
                    match msg.r#type() {
                        ZType::Zchat =>{
                            let zchat_msg = prost::bytes::Bytes::from(msg.clone().data);
                            let m = ZChat::decode(zchat_msg).unwrap();
                            let prost_clock = m.clock.unwrap();
                            let custom_clock_info: ClockInfo = (&prost_clock).into();
                            if arc_zchronod.lock().await.state.add(BTreeSet::from_iter(vec![m.message_data.clone()])) {
                                arc_zchronod.lock().await.storage.sinker_clock(String::from_utf8(msg.id.clone()).unwrap(), m.message_data, &custom_clock_info).await;
                                broadcast_state(arc_zchronod, msg, src).await;
                            }
                        }
                        _ => println!("TBD: now just support ZType::Z_TYPE_ZCHAT=4 todo!"),
                    }
                }
                ZAction::ZTypeRead => {
                    println!("TBD: action read todo!");
                    // todo!() & DB
                }
            }
        }
        ZIdentity::UTypeSer => {
            let parse_ret = serde_json::from_slice(&msg.data);
            match parse_ret {
                Err(_) => {
                    println!("\nErr: server_state please to use serde_json serialize");
                }
                Ok(input_state) => {
                    let (need_broadcast, merged) = arc_zchronod.lock().await.state.merge(&input_state);
                    if need_broadcast {
                        // self.broadcast_state().await;
                        broadcast_state(arc_zchronod.clone(), msg, src).await;
                    }
                    if merged {
                        let mut state_guard = arc_zchronod.lock().await;
                        let input_clock_info = &input_state.clock_info;
                        let state_clock_info = &state_guard.state.clock_info.clone();

                        state_guard.storage.sinker_merge_log(input_clock_info, state_clock_info).await;
                    }
                },
            }
        }
    }
}

pub(crate) async fn broadcast_state(arc_zchronod: Arc<Mutex<Zchronod>>, msg: ZMessage, src: SocketAddr) {
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
    arc_zchronod.lock().await.socket.send_to(&buf2, src).await.unwrap();
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