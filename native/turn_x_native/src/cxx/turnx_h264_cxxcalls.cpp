// RATIONALE: OpenH264 is C++ and is thus hard to interface in Rust.
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <list>
#include <memory>
#include <stdexcept>
#include <type_traits>
#include <wels/codec_api.h>

// Default bitrate in bits per second
constexpr uint32_t default_bitrate = 500'000;

// Throw a runtime error with the specified message if the condition is false
void turnx_assert(bool condition, const char *message) {
  if (!condition) {
    throw std::runtime_error{message};
  }
}

// Can be either YUV or H264
extern "C" struct turnx_h264_cxxcalls_frame {
  union p_t {
    // In the case of YUV data:
    uint8_t *data[3];
    
    // In the case of H264 data:
    uint8_t *buf;
  };
  p_t p;
  bool is_encoded;
  int32_t n_items;
};

extern "C" struct turnx_h264_cxxcalls_status { uint32_t bitrate; };

struct turnx_h264_cxxcalls_codec {
  ISVCEncoder *encoder = nullptr;
  ISVCDecoder *decoder = nullptr;

  // Probably should NOT convert to C++ smart pointers, Rust is managing these.
  std::list<turnx_h264_cxxcalls_frame *> frame_queue{};

  uint16_t w;
  uint16_t h;

  // w - Encoder width
  // h - Encoder height
  turnx_h264_cxxcalls_codec(uint16_t w, uint16_t h) : w{w}, h{h} {
    // === DECODER ===
    // create decoder
    turnx_assert(WelsCreateDecoder(&decoder) == 0, "creating decoder failed");
    turnx_assert(decoder != nullptr,
                 "creating decoder resulted in null pointer");

    // decoding parameters...
    auto decoding_parameters = SDecodingParam{};
    memset(&decoding_parameters, 0, sizeof(SDecodingParam));
    decoding_parameters.sVideoProperty.eVideoBsType = VIDEO_BITSTREAM_AVC;

    // initialize it
    turnx_assert(decoder->Initialize(&decoding_parameters) == 0,
                 "initializing decoder failed");

    // === ENCODER ===
    // create encoder
    turnx_assert(WelsCreateSVCEncoder(&encoder) == 0,
                 "creating encoder failed");
    turnx_assert(encoder != nullptr,
                 "creating encoder resulted in null pointer");

    // encoding parameters...
    auto encoding_parameters = SEncParamBase{};
    memset(&encoding_parameters, 0, sizeof(SEncParamBase));
    encoding_parameters.fMaxFrameRate = 30;
    encoding_parameters.iPicHeight = w;
    encoding_parameters.iPicWidth = h;
    encoding_parameters.iTargetBitrate = default_bitrate;
    // initialize encoder
    turnx_assert(encoder->Initialize(&encoding_parameters) == 0,
                 "initializing encoder failed");
  }
  ~turnx_h264_cxxcalls_codec() {
    // The unique_ptr construct should trigger the destructor for us
    encoder->Uninitialize();
    WelsDestroySVCEncoder(encoder);

    decoder->Uninitialize();
    WelsDestroyDecoder(decoder);
  }
};

auto codec = std::unique_ptr<turnx_h264_cxxcalls_codec>{nullptr};

// Static compile-time smoketests. These attempt to catch undefined behavior
// caused by Rust-C++ compatibility issues.
struct turnx_h264_cxxcalls_smoketest {
  // These structures shall be C-compatible.
  static_assert(std::is_pod<turnx_h264_cxxcalls_frame>::value == true,
                "this struct must be C-compatible to interface with Rust");
  static_assert(std::is_pod<turnx_h264_cxxcalls_status>::value == true,
                "this struct must be C-compatible to interface with Rust");
};

extern "C" void turnx_h264_cxxcalls_start(uint16_t w, uint16_t h) {
  turnx_assert(codec == nullptr,
               "codec must not have been initialized beforehand");
  codec = std::make_unique<turnx_h264_cxxcalls_codec>(w, h);
}
extern "C" void turnx_h264_cxxcalls_stop() {
  turnx_assert(codec != nullptr,
               "codec must have been initialized before teardown");
  codec = nullptr;
}
extern "C" void turnx_h264_cxxcalls_push(turnx_h264_cxxcalls_frame *enc_frame) {
  turnx_assert(enc_frame->is_encoded == true,
               "encoded frame must be encoded before pushing");
  codec->frame_queue.push_front(enc_frame);
}
extern "C" turnx_h264_cxxcalls_frame *turnx_h264_cxxcalls_pop() {
  turnx_assert(codec->frame_queue.size() > 0,
               "frame queue should not be empty");
  auto src_frame = codec->frame_queue.back();
  codec->frame_queue.pop_back();
  turnx_assert(src_frame->is_encoded == true,
               "frame queue should only have encoded frames");

  // === DECODER ===
  // zero out data
  SBufferInfo destination_buffer_info;
  memset(&destination_buffer_info, 0, sizeof(SBufferInfo));

  auto dec_frame = turnx_h264_cxxcalls_frame{.is_encoded = false, .n_items = 0};

  turnx_assert(codec->decoder->DecodeFrameNoDelay(
                   src_frame->p.buf, src_frame->n_items, dec_frame.p.data,
                   &destination_buffer_info) == 0,
               "can't decode this frame");

  // === ENCODER ===
  auto enc_frame =
      new turnx_h264_cxxcalls_frame{.is_encoded = true, .n_items = 0};

  auto source = SSourcePicture{};
  memset(&source, 0, sizeof(SSourcePicture));
  source.iPicWidth = codec->w;
  source.iPicHeight = codec->h;
  source.iStride[0] = (codec->w >> 0);
  source.iStride[1] = (codec->w >> 1);
  source.iStride[2] = (codec->w >> 1);
  source.iColorFormat = videoFormatI420;
#if 0
  // this may not be needed so we're just going to comment it out
  // additionally, performing memcpy may cause a lot of trouble for us
  memcpy(source.pData[0], dec_frame.p.data[0],
         source.iStride[0] * (codec->h >> 0));
  memcpy(source.pData[1], dec_frame.p.data[1],
         source.iStride[1] * (codec->h >> 1));
  memcpy(source.pData[2], dec_frame.p.data[2],
         source.iStride[2] * (codec->h >> 1));
#endif
  source.pData[0] = dec_frame.p.data[0];
  source.pData[1] = dec_frame.p.data[1];
  source.pData[2] = dec_frame.p.data[2];

  auto source_info = SFrameBSInfo{};
  memset(&source_info, 0, sizeof(SFrameBSInfo));

  turnx_assert(codec->encoder->EncodeFrame(&source, &source_info) ==
                   cmResultSuccess,
               "can't encode that frame");
  enc_frame->n_items = source_info.iFrameSizeInBytes;
  memcpy(enc_frame->p.buf, source_info.sLayerInfo->pBsBuf, enc_frame->n_items);
  return enc_frame;
}
extern "C" size_t turnx_h264_cxxcalls_size() {
  return codec->frame_queue.size();
}
