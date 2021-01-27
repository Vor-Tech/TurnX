mod vpx;
use serde::{Deserialize, Serialize};

const CREATE_SESSION: u8 = 0x11_u8;
const DELETE_SESSION: u8 = 0x12_u8;
const JOIN_SESSION: u8 = 0x21_u8;
const QUIT_SESSION: u8 = 0x22_u8;
const FRAME: u8 = 0x80_u8;

#[derive(Deserialize, Serialize)]
struct VPXFrame {
    command: u8,
    ident: u64,
    frame: Vec<Vec<u8>>,
    // TODO: Should we be checksumming?
}

fn main() {
    use erlang_port::{PortReceive, PortSend};

    let mut port = unsafe {
        use erlang_port::PacketSize;
        erlang_port::nouse_stdio(PacketSize::Four)
    };

    for inp in port.receiver.iter::<VPXFrame>() {
        let input: VPXFrame = inp;
        port.sender
            .reply::<Result<VPXFrame, VPXFrame>, VPXFrame, VPXFrame>(Ok(input))
    }
}
