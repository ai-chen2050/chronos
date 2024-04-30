use prost::Message;
use std::{collections::HashMap, net::UdpSocket};
use protos::{bussiness::ZChat, vlc::{self, Clock, ClockInfo}, zmessage::ZMessage};

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;

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

    // let message = "Hello, world!";
    let destination = "0.0.0.0:8050";

    socket.send_to(&buf3, destination)?;

    Ok(())
}