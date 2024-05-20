use prost::Message;
use protos::{
    bussiness::{
        GatewayType, QueryByMsgId, QueryByTableKeyId, QueryMethod, QueryResponse, ZGateway,
    },
    innermsg::{Action, Identity, Innermsg},
    zmessage::{ZMessage, ZType},
};
use std::{io::{Read, Write}, net::Shutdown};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let destination = "127.0.0.1:8050";
    let mut stream = TcpStream::connect(destination)?;

    // now support message: QueryByMsgid
    let msg_type = "by_msg_id";
    // let msg_type = "by_key_id_clockinfos";
    // let msg_type = "by_key_id_mergelogs";

    let mut data = Vec::new();
    if msg_type == "by_msg_id" {
        data = query_by_msg_id();
    } else if msg_type == "by_key_id_clockinfos" {
        data = query_by_key_id(GatewayType::ClockNode);
    } else if msg_type == "by_key_id_mergelogs" {
        data = query_by_key_id(GatewayType::MergeLog);
    }

    stream.write_all(&data)?;
    stream.shutdown(Shutdown::Write)?;

    // recv msg
    let mut buf = [0; 65535];
    let size = stream.read(&mut buf)?;

    if size > 0 {
        let msg = prost::bytes::Bytes::copy_from_slice(&buf[..size]);
        let inner_msg = Innermsg::decode(msg).unwrap();
        let response = QueryResponse::decode(inner_msg.message.unwrap().data.as_ref()).unwrap();
        println!("Received response: {:?}", response);
    } else {
        println!("No response received.");
    }

    Ok(())
}

fn query_by_msg_id() -> Vec<u8> {
    let msg_id = "todo1";
    let params = QueryByMsgId {
        msg_id: msg_id.to_owned(),
    };

    let mut buf1 = vec![];
    params.encode(&mut buf1).unwrap();

    let gateway = ZGateway {
        r#type: GatewayType::ClockNode.into(),
        method: QueryMethod::QueryByMsgid.into(),
        data: buf1,
    };

    let mut buf2 = vec![];
    gateway.encode(&mut buf2).unwrap();
    println!("buf2: {:?}", buf2);

    let p2p_msg = ZMessage {
        r#type: ZType::Gateway.into(),
        data: buf2,
        ..Default::default()
    };

    let inner_msg = Innermsg {
        identity: Identity::Client.into(),
        action: Action::Read.into(),
        message: Some(p2p_msg),
        ..Default::default()
    };

    let mut buf3 = vec![];
    inner_msg.encode(&mut buf3).unwrap();
    println!("buf3: {:?}", buf3);
    buf3
}

fn query_by_key_id(gw_type: GatewayType) -> Vec<u8> {
    let start_id = 0;
    let params = QueryByTableKeyId { last_pos: start_id };

    let gateway = ZGateway {
        r#type: gw_type.into(),
        method: QueryMethod::QueryByTableKeyid.into(),
        data: params.encode_to_vec(),
    };

    let p2p_msg = ZMessage {
        r#type: ZType::Gateway.into(),
        data: gateway.encode_to_vec(),
        ..Default::default()
    };

    let inner_msg = Innermsg {
        identity: Identity::Client.into(),
        action: Action::Read.into(),
        message: Some(p2p_msg),
        ..Default::default()
    };

    let mut buf3 = vec![];
    inner_msg.encode(&mut buf3).unwrap();
    println!("buf3: {:?}", buf3);
    buf3
}
