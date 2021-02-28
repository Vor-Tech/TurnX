// ABR: Adaptive Bitrate Encoding/Decoding
use std::collections::VecDeque;

extern crate gstreamer as gst;
use gst::prelude::*;
extern crate gstreamer_pbutils as gst_pbutils;
use gst_pbutils::prelude::*;

type PIDController = crate::turnx_gst::pid::PIDController<f64>;
type AVFrame = crate::turnx_gst::frame::AVFrame;

// PID controllers' knobs
const LAG_SENSE_PROPORTION: f64 = 0.02_f64;
const LAG_SENSE_INTEGRAL: f64 = 0.15_f64;
const LAG_SENSE_DERIVATIVE: f64 = 0.01_f64;
const LAG_SENSE_ENTRIES: usize = 4_usize;

// ABR Pipeline object
// IDEA: Make a FIFO, that's our lag sense tidbit.
struct Pipeline {
    // > Lag Sense PID Objects
    lag_sense_audio: PIDController,
    lag_sense_video: PIDController,

    // FIFOs for lag sense
    fifo_audio: VecDeque<AVFrame>,
    fifo_video: VecDeque<AVFrame>,

    // > Audio Source
    // > Video Source
    // VVVVVV
    // > Audio Decoder
    // > Video Decoder
    // VVVVVV
    // > Audio Resampler/Encoder
    // > Video Resampler/Encoder
    // VVVVVV
    // > Audio Sink
    // > Video Sink
    pipe: gst::Pipeline,
}

impl Pipeline {
    // Generate an ABR pipeline
    fn new(
        audio_min_rate: f64,
        audio_max_rate: f64,
        video_min_rate: f64,
        video_max_rate: f64,
        audio_fifo_depth: usize,
        video_fifo_depth: usize,
    ) -> Pipeline {
        Pipeline {
            lag_sense_audio: PIDController::new(
                audio_min_rate,
                audio_max_rate,
                LAG_SENSE_PROPORTION,
                LAG_SENSE_INTEGRAL,
                LAG_SENSE_DERIVATIVE,
                LAG_SENSE_ENTRIES,
                (audio_min_rate + audio_max_rate) / 2.0_f64,
                0.0_f64,
                0.0_f64,
            ),
            lag_sense_video: PIDController::new(
                video_min_rate,
                video_max_rate,
                LAG_SENSE_PROPORTION,
                LAG_SENSE_INTEGRAL,
                LAG_SENSE_DERIVATIVE,
                LAG_SENSE_ENTRIES,
                (video_min_rate + video_max_rate) / 2.0_f64,
                0.0_f64,
                0.0_f64,
            ),
        }
    }
}

impl Drop for Pipeline {
    // Destroy an ABR pipeline
    fn drop(&mut self) {}
}
