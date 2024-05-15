use prost::Message;
use std::net::UdpSocket;
use protos::{bussiness::{GatewayType, QueryByMsgId, QueryByTableKeyId, QueryMethod, QueryResponse, ZGateway}, innermsg::{Action, Identity, Innermsg}, zmessage::{ZMessage, ZType}};

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;

    // now support message: QueryByMsgid
    let msg_type = "by_msgid";
    // let msg_type = "by_keyid_clockinfos";
    // let msg_type = "by_keyid_mergelogs";

    let mut data = Vec::new();
    if msg_type == "by_msgid" {
        data = by_msgid();
    } else if msg_type == "by_keyid_clockinfos" {
        data = by_keyid(GatewayType::ClockNode);
    } else if msg_type == "by_keyid_mergelogs" {
        data = by_keyid(GatewayType::MergeLog);
    }
    
    let destination = "127.0.0.1:8050";
    socket.send_to(&data, destination)?;

    // recv msg
    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((size, _)) => {
            let msg = prost::bytes::Bytes::copy_from_slice(&buf[..size]);
            let  response= Innermsg::decode(msg).unwrap();
            let ret = QueryResponse::decode(response.message.unwrap().data.as_ref()).unwrap();
            println!("Received response: {:?}", ret);
        }
        Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
            println!("No response received.");
        }
        Err(err) => {
            eprintln!("Error receiving response: {}", err);
        }
    }

    Ok(())
}

fn by_msgid() -> Vec<u8> {
    let msg_id = "todo1";
    let params = QueryByMsgId {
        msg_id: msg_id.to_owned()
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

fn by_keyid(gw_type: GatewayType) -> Vec<u8> {
    let start_id = 0;
    let params = QueryByTableKeyId {
        last_pos: start_id,
    };
    
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

