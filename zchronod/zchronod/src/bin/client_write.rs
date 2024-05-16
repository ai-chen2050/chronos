use prost::Message;
use protos::{
    bussiness::ZChat,
    innermsg::{Action, Identity, Innermsg},
    vlc::{Clock, ClockInfo},
    zmessage::{ZMessage, ZType},
};
use std::{collections::{BTreeSet, HashMap}, net::Shutdown};
use std::io::{Read, Write};
use std::net::TcpStream;
use zchronod::zchronod::ServerState;

fn main() -> std::io::Result<()> {
    let destination = "127.0.0.1:8050";
    let mut stream = TcpStream::connect(destination)?;

    // now support message: client、full_sync_server
    let msg_type = "client"; // first step test
    // let msg_type = "full_sync_server";          // second step test

    let mut buf3 = Vec::new();
    if msg_type == "client" {
        buf3 = client_message();
    } else if msg_type == "full_sync_server" {
        buf3 = full_sync_server_message();
    }

    stream.write_all(&buf3)?;
    stream.shutdown(Shutdown::Write)?;

    // recv msg
    let mut buf = [0; 65535];
    let size = stream.read(&mut buf)?;

    if size > 0 {
        let msg = prost::bytes::Bytes::copy_from_slice(&buf[..size]);
        let response = Innermsg::decode(msg).unwrap();
        println!("Received response: {:?}", response);
    } else {
        println!("No response received.");
    }

    Ok(())
}

fn client_message() -> Vec<u8> {
    let mut values = HashMap::new();
    values.insert("one".to_owned(), 1);

    let clock = Some(Clock { values });
    let id = Vec::from("one");
    let message_id = Vec::from("message_id");
    let count = 0;
    let create_at = tools::helper::get_time_ms();

    let clock_info = ClockInfo {
        clock,
        id,
        message_id,
        count,
        create_at: create_at.try_into().unwrap(),
    };

    let zchat = ZChat {
        message_data: "hello".to_owned(),
        clock: Some(clock_info),
    };

    let mut buf2 = vec![];
    zchat.encode(&mut buf2).unwrap();

    let p2p_msg = ZMessage {
        id: Vec::from("intobytes"),
        from: Vec::from("msgfrom"),
        to: Vec::from("msg.to"),
        r#type: ZType::Zchat.into(),
        data: buf2,
        ..Default::default()
    };

    let inner_msg = Innermsg {
        identity: Identity::Client.into(),
        action: Action::Write.into(),
        message: Some(p2p_msg),
        ..Default::default()
    };

    let mut buf3 = vec![];
    inner_msg.encode(&mut buf3).unwrap();
    println!("buf: {:?}", buf3);
    buf3
}

fn full_sync_server_message() -> Vec<u8> {
    let mut server_state = ServerState::new("two".to_string());
    server_state.add(BTreeSet::from_iter(vec!["world".to_string()]));

    let serde_res = serde_json::to_string(&server_state);
    let serde_string = &serde_res.unwrap();
    let state_data = serde_string.as_bytes();

    let p2p_msg = ZMessage {
        id: Vec::from("intobytes"),
        from: Vec::from("msgfrom"),
        to: Vec::from("msg.to"),
        r#type: ZType::Zchat.into(),
        data: state_data.to_vec(),
        ..Default::default()
    };

    let inner_msg = Innermsg {
        identity: Identity::Server.into(),
        action: Action::Write.into(),
        message: Some(p2p_msg),
        ..Default::default()
    };

    let mut buf3 = vec![];
    inner_msg.encode(&mut buf3).unwrap();
    println!("buf: {:?}", buf3);
    buf3
}
