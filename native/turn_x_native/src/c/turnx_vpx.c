// RATIONALE: libvpx does not provide any mechanism for initialising some
// data structures. This makes it hard to use in Rust.
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include <vpx/vpx_decoder.h>
#include <vpx/vpx_encoder.h>

struct turnx_vpx_compander {
  uint32_t bitrate;

  vpx_codec_ctx_t cx_codec, dx_codec;
  vpx_codec_iface_t *cx_iface, *dx_iface;

  vpx_codec_pts_t frame;

  vpx_codec_enc_cfg_t cx_config;
  vpx_codec_dec_cfg_t dx_config;
};

// ============================================================================
// Create the compander (turnx_vpx_compander_create)
// ============================================================================
extern struct turnx_vpx_compander *turnx_vpx_compander_create(uint32_t w,
                                                              uint32_t h) {
  struct turnx_vpx_compander *codec =
      malloc(sizeof(struct turnx_vpx_compander));

  assert(vpx_codec_enc_config_default(codec->cx_iface, &codec->cx_config, 0) ==
         VPX_CODEC_OK);

  codec->cx_config.g_w = w; // Frame Width
  codec->cx_config.g_h = h; // Frame Height

  codec->cx_config.g_error_resilient = VPX_ERROR_RESILIENT_DEFAULT;
  codec->cx_config.rc_target_bitrate = 512;

  assert(vpx_codec_enc_init(&codec->cx_codec, codec->cx_iface,
                            &codec->cx_config,
                            VPX_CODEC_USE_FRAME_THREADING) == VPX_CODEC_OK);

  codec->dx_config.w = w;
  codec->dx_config.h = h;

  assert(vpx_codec_dec_init(&codec->dx_codec, codec->dx_iface,
                            &codec->dx_config,
                            VPX_CODEC_USE_FRAME_THREADING) == VPX_CODEC_OK);

  return codec; // Box this in Rust when done, thanks.
}

// ============================================================================
// Destroy the compander (turnx_vpx_compander_delete)
// ============================================================================
extern void turnx_vpx_compander_delete(struct turnx_vpx_compander *codec) {
  assert(vpx_codec_destroy(&codec->dx_codec) == VPX_CODEC_OK);
  assert(vpx_codec_destroy(&codec->cx_codec) == VPX_CODEC_OK);
  free(codec);
}
