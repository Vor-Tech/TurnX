fn main() {
    use erlang_port::{PortReceive, PortSend};

    let mut port = unsafe {
        use erlang_port::PacketSize;
        erlang_port::nouse_stdio(PacketSize::Four)
    };

    for in_str in port.receiver.iter::<String>() {
        let str = String::from(in_str);
        port.sender.reply::<Result<String, String>, String, String>(Ok(str))
    }
}