mod turnx_gst;

extern crate gstreamer as gst;

use std::{
    sync::{atomic::AtomicBool, Arc, RwLock},
    thread,
};

use tokio::sync::oneshot;
use tokio::task;
use tokio::time::{timeout, Duration};

type AVFrame = crate::turnx_gst::frame::AVFrame;
type AVPipeline = crate::turnx_gst::abr::Pipeline;

const HALT: u8 = 0x01_u8;
const HUNG_UP: u8 = 0x02_u8;
const AUDIO_ABR_QUALITY: u8 = 0x03_u8;
const VIDEO_ABR_QUALITY: u8 = 0x04_u8;
const AUDIO_FRAME_SEND: u8 = 0x05_u8;
const VIDEO_FRAME_SEND: u8 = 0x06_u8;
const AUDIO_FRAME_RECV: u8 = 0x07_u8;
const VIDEO_FRAME_RECV: u8 = 0x08_u8;
const PING: u8 = 0x80_u8;
const PONG: u8 = 0x81_u8;

#[tokio::main]
async fn main() {
    use erlang_port::{PortReceive, PortSend};

    assert!(
        gst::init().is_ok(),
        "Can't init GStreamer, are its dependencies installed?"
    );

    // ========================================================================
    //  RUNLOOP
    // ========================================================================
    let mut port = unsafe {
        use erlang_port::PacketSize;
        erlang_port::nouse_stdio(PacketSize::Four)
    };
    let mut running = true;
    while running {
        let mut item: Option<AVFrame> = None;
        while item.is_none() {
            // This isn't async but it'll block.
            thread::sleep(Duration::from_millis(1));
            item = port.receiver.receive::<AVFrame>()
        }
        let item_u = item.unwrap();
        // TODO: timeouts again
        match (item_u.command) {
            HALT => {
                running = false;
                port.sender
                    .reply::<Result<AVFrame, AVFrame>, AVFrame, AVFrame>(Ok(AVFrame {
                        command: HUNG_UP,
                        ident: item_u.ident,
                        frame: vec![vec![]],
                    }));
            }
            AUDIO_ABR_QUALITY => {}
            VIDEO_ABR_QUALITY => {}
            AUDIO_FRAME_RECV => {}
            VIDEO_FRAME_RECV => {}
            PING => {
                port.sender
                    .reply::<Result<AVFrame, AVFrame>, AVFrame, AVFrame>(Ok(AVFrame {
                        command: PONG,
                        ident: item_u.ident,
                        frame: vec![vec![]],
                    }));
            }
            _ => { panic!("Garbage command received, terminating...") }
        }
    }
    // The runloop SHOULD HAVE crashed if it was an error...
}
