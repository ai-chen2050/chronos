use std::net::SocketAddr;
use protos::innermsg::Innermsg;
use protos::vlc::ClockInfos as ProtoClockInfos;
use protos::vlc::{MergeLog as ProtoMergeLog, MergeLogs as ProtoMergeLogs};
use protos::zmessage::{ZMessage, ZType, ZMessages};
use prost::Message;
use crate::zchronod::ZchronodArc;
use tracing::*;
use crate::api::response::{
    make_query_response, respond_cli_query
};
use protos::bussiness::{
    GatewayType, QueryByMsgId, QueryByTableKeyId, QueryMethod, ZGateway
};

use super::response::clockinfo_to_proto;

pub async fn handle_cli_read_msg(arc_zchronod: ZchronodArc, inner_msg: Innermsg, p2p_msg: &ZMessage, src: SocketAddr) {
    match p2p_msg.r#type() {
        ZType::Gateway =>{
            let gateway_msg = prost::bytes::Bytes::from(p2p_msg.data.clone());
            let m = ZGateway::decode(gateway_msg).unwrap();
            match m.method() {
                QueryMethod::QueryByMsgid => query_by_msgid(arc_zchronod, inner_msg, m, src).await,
                QueryMethod::QueryByTableKeyid => query_by_table_keyid(arc_zchronod, inner_msg, m, src).await,
            }
        }
        _ => info!("Read: now just support ZType::Gateway = 3 todo!"),
    }
}

async fn query_by_msgid(arc_zchronod: ZchronodArc, inner_msg: Innermsg, m: ZGateway, src: SocketAddr) {
    let gateway_data = prost::bytes::Bytes::from(m.data.clone());
    let params = QueryByMsgId::decode(gateway_data);

    match params {
        Err(err) => {
            error!("QueryByMsgid params format error, err={:?}", err);
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

async fn query_clock_by_msgid(arc_zchronod: &ZchronodArc, query: &QueryByMsgId) -> (bool, String, Vec<u8>) {
    let clock_ret = arc_zchronod.storage.get_clock_by_msgid(&query.msg_id).await;
    let (success, message, clock_info) = match clock_ret {
        Ok(clock_info) => (true, String::new(), Some(clock_info)),
        Err(err) => (false, err.to_string(), None),
    };
    let proto_clock_info = clock_info.map(clockinfo_to_proto());
    let data = proto_clock_info.map(|info| info.encode_to_vec()).unwrap_or_else(Vec::new);
    (success, message, data)
}

async fn query_zmessage_by_msgid(arc_zchronod: &ZchronodArc, query: QueryByMsgId) -> (bool, String, Vec<u8>) {
    let msg_ret = arc_zchronod.storage.get_p2pmsg_by_msgid(&query.msg_id).await;
    let (success, message, z_message) = match msg_ret {
        Ok(clock_info) => (true, String::new(), Some(clock_info)),
        Err(err) => (false, err.to_string(), None),
    };
    let data = z_message.map(|msg| msg.encode_to_vec()).unwrap_or_else(Vec::new);
    (success, message, data)
}

pub async fn query_by_table_keyid(arc_zchronod: ZchronodArc, inner_msg: Innermsg, m: ZGateway, src: SocketAddr) {
    let gateway_data = prost::bytes::Bytes::from(m.data.clone());
    let params = QueryByTableKeyId::decode(gateway_data);
    let batch_num = arc_zchronod.config.read_maximum;
    match params {
        Err(err) => {
            error!("QueryByTableKeyid params format error, err={:?}", err);
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

async fn query_clockinfo_batch(arc_zchronod: &ZchronodArc, query: QueryByTableKeyId, batch_num: u64) -> (bool, String, Vec<u8>) {
    let clocks_ret = arc_zchronod.storage.get_clocks_by_keyid(query.last_pos, batch_num).await;

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

async fn query_zmessage_batch(arc_zchronod: &ZchronodArc, query: QueryByTableKeyId, batch_num: u64) -> (bool, String, Vec<u8>) {
    let zmessages_ret = arc_zchronod.storage.get_zmessages_by_keyid(query.last_pos, batch_num).await;

    let (success, message, zmessages) = match zmessages_ret {
        Ok(clock_infos) => (true, String::new(), Some(clock_infos)),
        Err(err) => (false, err.to_string(), None),
    };

    let data = zmessages
        .map(|z_messages| ZMessages{messages: z_messages}.encode_to_vec())
        .unwrap_or_else(Vec::new);
    (success, message, data)
}

async fn query_mergelog_batch(arc_zchronod: &ZchronodArc, query: QueryByTableKeyId, batch_num: u64) -> (bool, String, Vec<u8>) {
    let mergelogs_ret = arc_zchronod.storage.get_mergelogs_by_keyid(query.last_pos, batch_num).await;

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