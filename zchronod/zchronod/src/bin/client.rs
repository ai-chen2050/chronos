use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;

    let message = "Hello, world!";
    let destination = "255.255.255.255:12345";

    socket.send_to(message.as_bytes(), destination)?;

    Ok(())
}