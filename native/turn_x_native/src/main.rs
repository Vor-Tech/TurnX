mod turnx_gst;

extern crate gstreamer as gst;
use gst::prelude::*;
extern crate gstreamer_pbutils as gst_pbutils;
use gst_pbutils::prelude::*;

use tokio::sync;
use tokio::task;
use tokio::time::{sleep, timeout, Duration};
use eetf::{Term, Atom};

type AVFrame = crate::turnx_gst::frame::AVFrame;
type AVFrameOpt = Option<AVFrame>;

const HALT: u8 = 0x01_u8;
const AUDIO_ABR_QUALITY: u8 = 0x02_u8;
const VIDEO_ABR_QUALITY: u8 = 0x03_u8;
const AUDIO_FRAME_SEND: u8 = 0x04_u8;
const VIDEO_FRAME_SEND: u8 = 0x05_u8;
const AUDIO_FRAME_RECV: u8 = 0x06_u8;
const VIDEO_FRAME_RECV: u8 = 0x07_u8;
const PING: u8 = 0x80_u8;
const PONG: u8 = 0x81_u8;

#[tokio::main]
async fn main() {
    use erlang_port::{PortReceive, PortSend};

    let mut port = unsafe {
        use erlang_port::PacketSize;
        erlang_port::nouse_stdio(PacketSize::Four)
    };

    assert!(
        gst::init().is_ok(),
        "Can't init Gstreamer, are its dependencies installed?"
    );

    // ========================================================================
    //  RUNLOOP
    // ========================================================================
    loop {
        let join = timeout(
            Duration::from_secs(5),
            task::spawn_blocking(move || {
                let mut item: Option<AVFrame> = None;
                while item.is_none() {
                    sleep(Duration::from_millis(1));
                    item = port.receiver.receive::<AVFrame>()
                };
                item.unwrap()
            })
        ).await;
        assert!(join.is_ok(), "AV task in runloop has crashed, aborting");
        
        let result = join.unwrap();
        
        // The runloop SHOULD HAVE crashed if it was an error... 
        port.sender
            .reply::<Result<AVFrameOpt, AVFrameOpt>, AVFrameOpt, AVFrameOpt>(Ok(result.ok()));
    }
}
