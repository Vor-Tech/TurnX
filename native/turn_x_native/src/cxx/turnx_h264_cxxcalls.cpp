// RATIONALE: OpenH264 is C++ and is thus hard to interface in Rust.
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <list>
#include <memory>
#include <type_traits>
#include <wels/codec_api.h>

// Default bitrate in kilobits per second
constexpr uint32_t default_bitrate = 500;

// Can be either: YUV ... or ... H264

extern "C" struct turnx_h264_cxxcalls_frame {
  union p {
    // In the case of H264 data:
    uint8_t *buf;

    // In the case of YUV data:
    uint8_t *data[3];
  };
  bool is_encoded;
  int32_t n_items;
};

extern "C" struct turnx_h264_cxxcalls_status { uint32_t bitrate; };

struct turnx_h264_cxxcalls_codec {
  ISVCDecoder *decoder;
  SBufferInfo destination_buffer_info;
  std::list<turnx_h264_cxxcalls_frame *> frame_queue{};

  // enc_w - Encoder width
  // enc_h - Encoder height
  // dec_w - Decoder width
  // dec_h - Decoder height
  turnx_h264_cxxcalls_codec(uint16_t enc_w, uint16_t enc_h, uint16_t dec_w,
                            uint16_t dec_h) {

    // === DECODER ===
    // zero out data
    memset(&destination_buffer_info, 0, sizeof(SBufferInfo));

    // create decoder
    assert(WelsCreateDecoder(&decoder) == 0);

    // decoding parameters...
    SDecodingParam decoding_parameters = {0};
    decoding_parameters.sVideoProperty.eVideoBsType = VIDEO_BITSTREAM_AVC;

    // initialize it
    assert(decoder->Initialize(&decoding_parameters) == 0);
  }
};

std::unique_ptr<turnx_h264_cxxcalls_codec> codec{nullptr};

// Static compile-time smoketests. These attempt to catch undefined behavior
// caused by Rust-C++ compatibility issues.
struct turnx_h264_cxxcalls_smoketest {
  // These structures shall be C-compatible.
  static_assert(std::is_pod<turnx_h264_cxxcalls_frame>::value == true);
  static_assert(std::is_pod<turnx_h264_cxxcalls_status>::value == true);
};

extern "C" void turnx_h264_cxxcalls_start() {
  assert(codec == nullptr);
  codec = std::make_unique<turnx_h264_cxxcalls_codec>();
}
extern "C" void turnx_h264_cxxcalls_stop() {
  assert(codec != nullptr);
  codec = nullptr;
}
extern "C" void turnx_h264_cxxcalls_push(turnx_h264_cxxcalls_frame *enc_frame) {
  assert(enc_frame->is_encoded == true);
  codec->frame_queue.push_front(enc_frame);
}
extern "C" turnx_h264_cxxcalls_frame *turnx_h264_cxxcalls_pop() {
  // === DECODER ===
  assert(codec->frame_queue.size() > 0);
  turnx_h264_cxxcalls_frame *dec_frame = codec->frame_queue.back();
  codec->frame_queue.pop_back();
  assert(dec_frame->is_encoded == false);

  // === ENCODER ===
  turnx_h264_cxxcalls_frame *enc_frame =
      new turnx_h264_cxxcalls_frame{.is_encoded = true, .n_items = 0};

  return enc_frame;
}
extern "C" size_t turnx_h264_cxxcalls_size() {
  return codec->frame_queue.size();
}
