use prost::Message;
use std::{collections::{BTreeSet, HashMap}, net::UdpSocket};
use protos::{bussiness::ZChat, vlc::{self, Clock, ClockInfo}, zmessage::ZMessage};
use Zchronod::zchronod::ServerState;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;

    // now support message: client、full_sync_server
    let message_type = "client";
    let message_type = "full_sync_server";

    let mut buf3 = Vec::new();
    if message_type == "client" {
        buf3 = client_message();
    } else if message_type == "full_sync_server" {
        buf3 = full_sync_server_message();
    }
    
    // let message = "Hello, world!";
    let destination = "0.0.0.0:8050";

    socket.send_to(&buf3, destination)?;

    // recv msg
    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((size, _)) => {
            let msg = prost::bytes::Bytes::copy_from_slice(&buf[..size]);
            let  response= ZMessage::decode(msg).unwrap();
            println!("Received response: {:?}", response);
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
    println!("buf2: {:?}", buf2);
    
    let msg = ZMessage {
        id: Vec::from("intobytes"),
        from: Vec::from("msgfrom"),
        to: Vec::from("msg.to"),
        r#type: 4,
        action: 1,
        data: buf2,
        identity: 0,
        ..Default::default()
    };
    
    let mut buf3 = vec![];
    msg.encode(&mut buf3).unwrap();
    println!("buf: {:?}", buf3);
    buf3
}

fn full_sync_server_message() -> Vec<u8> {
    let mut server_state = ServerState::new("two".to_string());
    server_state.add(BTreeSet::from_iter(vec!["world".to_string()]));

    let serde_res = serde_json::to_string(&server_state);
    let serde_string = &serde_res.unwrap();
    let state_data = serde_string.as_bytes();
    
    let msg = ZMessage {
        id: Vec::from("intobytes"),
        from: Vec::from("msgfrom"),
        to: Vec::from("msg.to"),
        r#type: 4,
        action: 1,
        data: state_data.to_vec(),
        identity: 1,
        ..Default::default()
    };
    
    let mut buf3 = vec![];
    msg.encode(&mut buf3).unwrap();
    println!("buf: {:?}", buf3);
    buf3
}