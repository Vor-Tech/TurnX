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
pub struct Pipeline {
    // > Lag Sense PID Objects
    lag_sense_audio: PIDController,
    lag_sense_video: PIDController,

    // FIFOs for lag sense
    fifo_audio: VecDeque<AVFrame>,
    fifo_video: VecDeque<AVFrame>,
    fifo_size_audio: usize,
    fifo_size_video: usize,

    pipe: gst::Pipeline,

    pipe_av_src: gst::Element,
    pipe_rtp_demux: gst::Element,
    pipe_audio_payloadunpack: gst::Element,
    pipe_audio_decoder: gst::Element,
    pipe_audio_encoder: gst::Element,
    pipe_audio_payloadpack: gst::Element,
    pipe_video_payloadunpack: gst::Element,
    pipe_video_decoder: gst::Element,
    pipe_video_encoder: gst::Element,
    pipe_video_payloadpack: gst::Element,
    pipe_rtp_mux: gst::Element,
    pipe_av_sink: gst::Element,
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
        // Determine baseline median values.
        let audio_median = (audio_min_rate + audio_max_rate) / 2.0_f64;
        let video_median = (video_min_rate + video_max_rate) / 2.0_f64;

        let this = Pipeline {
            // PIPELINE
            pipe: gst::Pipeline::new(None),

            pipe_av_src: gst::ElementFactory::make("appsrc", None)
                .expect("Can't init application AV source"),
            pipe_rtp_demux: gst::ElementFactory::make("rtpptdemux", None)
                .expect("Can't init RTP demultiplexer, do you have `gst-plugins-good` installed?"),

            pipe_audio_payloadunpack: gst::ElementFactory::make("rtpopusdepay", None).expect(
                "Can't init RTP->Opus de-payloader, do you have `gst-plugins-good` installed?",
            ),
            pipe_audio_decoder: gst::ElementFactory::make("opusdec", None)
                .expect("Can't init Opus decoder, do you have `gst-plugins-base` installed?"),
            pipe_audio_encoder: gst::ElementFactory::make("opusenc", None)
                .expect("Can't init Opus encoder, do you have `gst-plugins-base` installed?"),
            pipe_audio_payloadpack: gst::ElementFactory::make("rtpopuspay", None).expect(
                "Can't init Opus->RTP payloader, do you have `gst-plugins-good` installed?",
            ),

            pipe_video_payloadunpack: gst::ElementFactory::make("rtph264depay", None).expect(
                "Can't init RTP->H264 de-payloader, do you have `gst-plugins-good` installed?",
            ),
            pipe_video_decoder: gst::ElementFactory::make("openh264dec", None)
                .expect("Can't init OpenH264 decoder, do you have `gst-plugins-bad` installed?"),
            pipe_video_encoder: gst::ElementFactory::make("openh264enc", None)
                .expect("Can't init OpenH264 encoder, do you have `gst-plugins-bad` installed?"),
            pipe_video_payloadpack: gst::ElementFactory::make("rtph264pay", None).expect(
                "Can't init H264->RTP payloader, do you have `gst-plugins-good` installed?",
            ),

            pipe_rtp_mux: gst::ElementFactory::make("rtpmux", None)
                .expect("Can't init RTP demultiplexer, do you have `gst-plugins-good` installed?"),
            pipe_av_sink: gst::ElementFactory::make("appsink", None)
                .expect("Can't init application AV sink"),

            fifo_audio: vec![].into_iter().collect(),
            fifo_video: vec![].into_iter().collect(),
            fifo_size_audio: audio_fifo_depth,
            fifo_size_video: video_fifo_depth,
            lag_sense_audio: PIDController::new(
                audio_min_rate,
                audio_max_rate,
                LAG_SENSE_PROPORTION,
                LAG_SENSE_INTEGRAL,
                LAG_SENSE_DERIVATIVE,
                LAG_SENSE_ENTRIES,
                audio_median,
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
                video_median,
                0.0_f64,
                0.0_f64,
            ),
        };

        // temporary error message that should be changed
        this.pipe
            .add_many(&[
                &this.pipe_av_src,
                &this.pipe_rtp_demux,
                &this.pipe_audio_payloadunpack,
                &this.pipe_audio_decoder,
                &this.pipe_audio_encoder,
                &this.pipe_audio_payloadpack,
                &this.pipe_video_payloadunpack,
                &this.pipe_video_decoder,
                &this.pipe_video_encoder,
                &this.pipe_video_payloadpack,
                &this.pipe_rtp_mux,
                &this.pipe_av_sink,
            ])
            .expect("can't link, check line numbers and contact your local guru");

        gst::Element::link_many(&[&this.pipe_av_src, &this.pipe_rtp_demux])
            .expect("can't link, check line numbers and contact your local guru");

        gst::Element::link_many(&[&this.pipe_rtp_demux, &this.pipe_audio_payloadunpack])
            .expect("can't link, check line numbers and contact your local guru");
        gst::Element::link_many(&[
            &this.pipe_audio_payloadunpack,
            &this.pipe_audio_decoder,
            &this.pipe_audio_encoder,
            &this.pipe_audio_payloadpack,
        ])
        .expect("can't link, check line numbers and contact your local guru");
        gst::Element::link_many(&[&this.pipe_audio_payloadpack, &this.pipe_rtp_mux])
            .expect("can't link, check line numbers and contact your local guru");

        gst::Element::link_many(&[&this.pipe_rtp_demux, &this.pipe_video_payloadunpack])
            .expect("can't link, check line numbers and contact your local guru");
        gst::Element::link_many(&[
            &this.pipe_video_payloadunpack,
            &this.pipe_video_decoder,
            &this.pipe_video_encoder,
            &this.pipe_video_payloadpack,
        ])
        .expect("can't link, check line numbers and contact your local guru");
        gst::Element::link_many(&[&this.pipe_video_payloadpack, &this.pipe_rtp_mux])
            .expect("can't link, check line numbers and contact your local guru");

        gst::Element::link_many(&[&this.pipe_rtp_mux, &this.pipe_av_sink])
            .expect("can't link, check line numbers and contact your local guru");

        // Finally, return our Pipeline object.
        this
    }
}

impl Drop for Pipeline {
    // Destroy an ABR pipeline
    fn drop(&mut self) {}
}
