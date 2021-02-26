mod turnx_gst;

extern crate gstreamer as gst;
use gst::prelude::*;
extern crate gstreamer_pbutils as gst_pbutils;
use gst_pbutils::prelude::*;

use serde::{Deserialize, Serialize};

const CREATE_USER: u8 = 0x11_u8;
const DELETE_USER: u8 = 0x12_u8;
const RAISE_ABR_QUALITY: u8 = 0x21_u8;
const LOWER_ABR_QUALITY: u8 = 0x22_u8;
const FRAME: u8 = 0x80_u8;

#[derive(Deserialize, Serialize)]
struct VideoFrame {
    command: u8,
    ident: i64,
    frame: Vec<Vec<u8>>,
    // TODO: Should we be checksumming?
}

fn main() {
    use erlang_port::{PortReceive, PortSend};

    let mut port = unsafe {
        use erlang_port::PacketSize;
        erlang_port::nouse_stdio(PacketSize::Four)
    };

    assert_eq!(
        gst::init().is_ok(),
        true,
        "Can't init Gstreamer, are its dependencies installed?"
    );

    for inp in port.receiver.iter::<VideoFrame>() {
        let input: VideoFrame = inp;
        port.sender
            .reply::<Result<VideoFrame, VideoFrame>, VideoFrame, VideoFrame>(Ok(input))
    }
}
