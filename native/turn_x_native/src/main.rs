mod turnx_gst;

extern crate gstreamer as gst;

use std::{
    sync::{Arc, RwLock},
    thread,
};
use tokio::task;
use tokio::time::{timeout, Duration};

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

    assert!(
        gst::init().is_ok(),
        "Can't init GStreamer, are its dependencies installed?"
    );

    // ========================================================================
    //  RUNLOOP
    // ========================================================================
    let port = Arc::new(RwLock::new(unsafe {
        use erlang_port::PacketSize;
        erlang_port::nouse_stdio(PacketSize::Four)
    }));
    loop {
        let ref_port = Arc::clone(&port);
        assert!(
            timeout(
                Duration::from_secs(3),
                task::spawn_blocking(move || {
                    if let Ok(mut port_w) = ref_port.write() {
                        let mut item: Option<AVFrame> = None;
                        while item.is_none() {
                            // This isn't async but it'll block.
                            thread::sleep(Duration::from_millis(1));
                            item = port_w.receiver.receive::<AVFrame>()
                        }
                        // Unwrapping is safe here.
                        port_w
                            .sender
                            .reply::<Result<AVFrame, AVFrame>, AVFrame, AVFrame>(Ok(item.unwrap()));
                    } else {
                        panic!("Can't lock port for write")
                    }
                })
            )
            .await
            .is_ok(),
            "Runloop crashed"
        );
    }
    // The runloop SHOULD HAVE crashed if it was an error...
}
