use std::net::SocketAddr;
use std::{cmp, collections::BTreeSet, sync::Arc};
use node_api::config::ZchronodConfig;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use websocket::ReceiveMessage;
use crate::{node_factory::ZchronodFactory, storage::Storage, vlc::Clock};
use serde::{Deserialize, Serialize};
use prost::Message;
use crate::vlc::ClockInfo;
use protos::innermsg::{Action, Identity, Innermsg, PushType};
use protos::vlc::{ClockInfo as ProtoClockInfo, ClockInfos as ProtoClockInfos};
use protos::vlc::{MergeLog as ProtoMergeLog, MergeLogs as ProtoMergeLogs};
use protos::vlc::Clock as ProtoClock;
use protos::zmessage::{ZMessage, ZType, ZMessages};
use protos::bussiness::{GatewayType, QueryByMsgId, QueryByTableKeyId, QueryMethod, QueryResponse, ZChat, ZGateway};

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
            clock_info: ClockInfo::new(Clock::new(), String::new(), node_id.clone(), "".to_owned(), 0),
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

pub(crate) async fn p2p_event_loop(arc_zchronod: Arc<RwLock<Zchronod>>) {
    println!("Now p2p udp listen on : {}", arc_zchronod.read().await.config.inner_p2p);
    loop {
        let mut buf = [0; 65535];
        let (n, src) = arc_zchronod.read().await.socket.recv_from(&mut buf).await.unwrap();
        let msg = prost::bytes::Bytes::copy_from_slice(&buf[..n]);
        if let Ok(m) = Innermsg::decode(msg) {
            println!("-> Received message from identity: {:?}, action: {:?}", m.identity(), m.action());
            handle_msg(arc_zchronod.clone(), m, src).await;
        } else {
            println!("No action, only support innermsg type between vlc & p2p modules at now");
        }
    }
}

pub(crate) async fn handle_msg(arc_zchronod: Arc<RwLock<Zchronod>>, inner_msg: Innermsg, src: SocketAddr) {
    if let Some(p2p_msg) = &inner_msg.clone().message {
        match inner_msg.identity() {
            Identity::Client => handle_cli_msg(inner_msg, p2p_msg, arc_zchronod, src).await,
            Identity::Server => handle_srv_msg(inner_msg, p2p_msg, arc_zchronod, src).await,
            Identity::Init => {todo!()},
        }
    } else {
        println!("p2p_msg is empty, no action triggered!");
    }
}

async fn handle_srv_msg(inner_msg: Innermsg, p2p_msg: &ZMessage, arc_zchronod: Arc<RwLock<Zchronod>>, src: SocketAddr) {
    let parse_ret = serde_json::from_slice(&p2p_msg.data);
    match parse_ret {
        Err(_) => {
            println!("Err: server_state please to use serde_json serialize");
        }
        Ok(input_state) => {
            let (need_broadcast, merged) = arc_zchronod.write().await.state.merge(&input_state);
            if need_broadcast {
                broadcast_srv_state(arc_zchronod.clone(), inner_msg, src).await;
            }
            if merged {
                let mut state_guard = arc_zchronod.write().await;
                let input_clock_info = &input_state.clock_info;
                let state_clock_info = &state_guard.state.clock_info.clone();
    
                state_guard.storage.sinker_merge_log(input_clock_info, state_clock_info).await;
            }
        },
    }
}

async fn handle_cli_msg(inner_msg: Innermsg, p2p_msg: &ZMessage, arc_zchronod: Arc<RwLock<Zchronod>>, src: SocketAddr) {
    match inner_msg.action() {
        Action::Write => handle_cli_write_msg(arc_zchronod, inner_msg, p2p_msg, src).await,
        Action::Read => handle_cli_read_msg(arc_zchronod, inner_msg, p2p_msg, src).await,
        _ => {}
    }
}

async fn handle_cli_write_msg(arc_zchronod: Arc<RwLock<Zchronod>>, inner_msg: Innermsg, p2p_msg: &ZMessage, src: SocketAddr) {
    match p2p_msg.r#type() {
        ZType::Zchat =>{
            let zchat_msg = prost::bytes::Bytes::from(p2p_msg.data.clone());
            let m = ZChat::decode(zchat_msg).unwrap();
            let prost_clock = m.clock.unwrap();
            let custom_clock_info: ClockInfo = (&prost_clock).into();
            if arc_zchronod.write().await.state.add(BTreeSet::from_iter(vec![m.message_data.clone()])) {
                arc_zchronod.write().await.storage.sinker_clock(String::from_utf8(p2p_msg.id.clone()).unwrap(), m.message_data, &custom_clock_info).await;
                arc_zchronod.write().await.storage.sinker_zmessage(p2p_msg.clone()).await;
                broadcast_srv_state(arc_zchronod, inner_msg, src).await;
            }
        }
        _ => println!("Write: now just support ZType::Zchat = 4!"),
    }
}

async fn handle_cli_read_msg(arc_zchronod: Arc<RwLock<Zchronod>>, inner_msg: Innermsg, p2p_msg: &ZMessage, src: SocketAddr) {
    match p2p_msg.r#type() {
        ZType::Gateway =>{
            let gateway_msg = prost::bytes::Bytes::from(p2p_msg.data.clone());
            let m = ZGateway::decode(gateway_msg).unwrap();
            match m.method() {
                QueryMethod::QueryByMsgid => query_by_msgid(arc_zchronod, inner_msg, m, src).await,
                QueryMethod::QueryByTableKeyid => query_by_table_keyid(arc_zchronod, inner_msg, m, src).await,
            }
        }
        _ => println!("Read: now just support ZType::Gateway = 3 todo!"),
    }
}

async fn query_by_msgid(arc_zchronod: Arc<RwLock<Zchronod>>, inner_msg: Innermsg, m: ZGateway, src: SocketAddr) {
    let gateway_data = prost::bytes::Bytes::from(m.data.clone());
    let params = QueryByMsgId::decode(gateway_data);

    match params {
        Err(err) => {
            eprintln!("QueryByMsgid params format error, err={:?}", err);
            let response = make_query_response(false, format!("Params format error: {:?}", err), &vec![]);
            respond_cli_query(arc_zchronod, inner_msg, &response.encode_to_vec(), src).await;
        }
        Ok(query) => {
            let (success, message, data) = match m.r#type() {
                GatewayType::ClockNode => query_clock_by_msgid(&arc_zchronod, &query).await,
                GatewayType::ZMessage => query_zmessage_by_msgid(&arc_zchronod, query).await,
                _ => (false, "Not support gateway_type".to_string(), Vec::new()),
            };

            let response = make_query_response(success, message, &data);
            respond_cli_query(arc_zchronod, inner_msg, &response.encode_to_vec(), src).await;
        }
    }
}

async fn query_clock_by_msgid(arc_zchronod: &Arc<RwLock<Zchronod>>, query: &QueryByMsgId) -> (bool, String, Vec<u8>) {
    let clock_ret = arc_zchronod.read().await.storage.get_clock_by_msgid(&query.msg_id).await;
    let (success, message, clock_info) = match clock_ret {
        Ok(clock_info) => (true, String::new(), Some(clock_info)),
        Err(err) => (false, err.to_string(), None),
    };
    let proto_clock_info = clock_info.map(clockinfo_to_proto());
    let data = proto_clock_info.map(|info| info.encode_to_vec()).unwrap_or_else(Vec::new);
    (success, message, data)
}

async fn query_zmessage_by_msgid(arc_zchronod: &Arc<RwLock<Zchronod>>, query: QueryByMsgId) -> (bool, String, Vec<u8>) {
    let msg_ret = arc_zchronod.read().await.storage.get_p2pmsg_by_msgid(&query.msg_id).await;
    let (success, message, z_message) = match msg_ret {
        Ok(clock_info) => (true, String::new(), Some(clock_info)),
        Err(err) => (false, err.to_string(), None),
    };
    let data = z_message.map(|msg| msg.encode_to_vec()).unwrap_or_else(Vec::new);
    (success, message, data)
}

pub async fn query_by_table_keyid(arc_zchronod: Arc<RwLock<Zchronod>>, inner_msg: Innermsg, m: ZGateway, src: SocketAddr) {
    let gateway_data = prost::bytes::Bytes::from(m.data.clone());
    let params = QueryByTableKeyId::decode(gateway_data);
    let batch_num = arc_zchronod.read().await.config.read_maximum;
    match params {
        Err(err) => {
            eprintln!("QueryByTableKeyid params format error, err={:?}", err);
            let response = make_query_response(false, format!("Params format error: {:?}", err), &vec![]);
            respond_cli_query(arc_zchronod, inner_msg, &response.encode_to_vec(), src).await;
        }
        Ok(query) => {
            let (success, message, data) = match m.r#type() {
                GatewayType::ClockNode => query_clockinfo_batch(&arc_zchronod, query, batch_num).await,
                GatewayType::MergeLog => query_mergelog_batch(&arc_zchronod, query, batch_num).await,
                GatewayType::ZMessage => query_zmessage_batch(&arc_zchronod, query, batch_num).await,
                _ => (false, "Not support gateway_type".to_string(), Vec::new()),
            };
            let response = make_query_response(success, message, &data);
            respond_cli_query(arc_zchronod, inner_msg, &response.encode_to_vec(), src).await;
        }
    }
}

async fn query_clockinfo_batch(arc_zchronod: &Arc<RwLock<Zchronod>>, query: QueryByTableKeyId, batch_num: u64) -> (bool, String, Vec<u8>) {
    let clocks_ret = arc_zchronod.read().await.storage.get_clocks_by_keyid(query.last_pos, batch_num).await;

    let (success, message, clock_infos) = match clocks_ret {
        Ok(clock_infos) => (true, String::new(), Some(clock_infos)),
        Err(err) => (false, err.to_string(), None),
    };

    let proto_clock_infos = clock_infos.map(|clock_infos| {
        clock_infos
            .into_iter()
            .map(clockinfo_to_proto())
            .collect::<Vec<_>>()
    });

    let data = proto_clock_infos
        .map(|infos| ProtoClockInfos{clock_infos: infos}.encode_to_vec())
        .unwrap_or_else(Vec::new);
    (success, message, data)
}

async fn query_zmessage_batch(arc_zchronod: &Arc<RwLock<Zchronod>>, query: QueryByTableKeyId, batch_num: u64) -> (bool, String, Vec<u8>) {
    let zmessages_ret = arc_zchronod.read().await.storage.get_zmessages_by_keyid(query.last_pos, batch_num).await;

    let (success, message, zmessages) = match zmessages_ret {
        Ok(clock_infos) => (true, String::new(), Some(clock_infos)),
        Err(err) => (false, err.to_string(), None),
    };

    let data = zmessages
        .map(|z_messages| ZMessages{messages: z_messages}.encode_to_vec())
        .unwrap_or_else(Vec::new);
    (success, message, data)
}

async fn query_mergelog_batch(arc_zchronod: &Arc<RwLock<Zchronod>>, query: QueryByTableKeyId, batch_num: u64) -> (bool, String, Vec<u8>) {
    let mergelogs_ret = arc_zchronod.read().await.storage.get_mergelogs_by_keyid(query.last_pos, batch_num).await;

    let (success, message, merge_logs) = match mergelogs_ret {
        Ok(merge_log) => (true, String::new(), Some(merge_log)),
        Err(err) => (false, err.to_string(), None),
    };

    let proto_merge_logs = merge_logs.map(|merge_logs| {
        merge_logs
            .into_iter()
            .map(|merge_log| ProtoMergeLog {
                from_id: merge_log.from_id.into(),
                to_id: merge_log.to_id.into(),
                start_count: merge_log.start_count as u64,
                end_count: merge_log.end_count as u64,
                s_clock_hash: merge_log.s_clock_hash.into(),
                e_clock_hash: merge_log.e_clock_hash.into(),
                merge_at: merge_log.merge_at as u64,
            })
            .collect::<Vec<_>>()
    });

    let data = proto_merge_logs
        .map(|logs| ProtoMergeLogs{merge_logs: logs}.encode_to_vec())
        .unwrap_or_else(Vec::new);
    (success, message, data)
}

pub(crate) async fn broadcast_srv_state(arc_zchronod: Arc<RwLock<Zchronod>>, mut inner: Innermsg, src: SocketAddr) {
    let serde_res = serde_json::to_string(&arc_zchronod.read().await.state);
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
    println!("<- Bd-srv-state: {:?}", inner);
    arc_zchronod.write().await.socket.send_to(&buf, src).await.unwrap();
}

pub(crate) async fn respond_cli_query(arc_zchronod: Arc<RwLock<Zchronod>>, mut inner: Innermsg, p2p_data: &[u8] ,src: SocketAddr) {
    let mut p2p_msg = inner.message.unwrap();
    p2p_msg.data = p2p_data.to_vec();
    inner.message = Some(p2p_msg);
    inner.identity = Identity::Server.into();
    inner.action = Action::ReadReply.into();
    inner.push_type = PushType::Direct.into();

    let mut buf = vec![];
    inner.encode(&mut buf).unwrap();
    println!("<- Response: {:?}", inner);
    arc_zchronod.write().await.socket.send_to(&buf, src).await.unwrap();
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

pub fn make_query_response(success: bool, reason: String, data: &[u8]) -> QueryResponse {
    let response = QueryResponse{
        success,
        reason,
        data: data.to_vec(),
    };

    response
}

fn clockinfo_to_proto() -> impl FnMut(ClockInfo) -> ProtoClockInfo {
    move |clock_info| {
        ProtoClockInfo {
            clock: Some(ProtoClock {
                values: clock_info.clock.values.into_iter().map(|(k, v)| (k, v as u64)).collect(),
            }),
            node_id: clock_info.node_id.into(),
            clock_hash: clock_info.clock_hash.into(),
            message_id: clock_info.message_id.into(),
            count: clock_info.count as u64,
            create_at: clock_info.create_at as u64,
        }
    }
}