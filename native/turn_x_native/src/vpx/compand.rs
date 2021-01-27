// Compander: Compressor/Expander
// Compress because bandwidth is scarce
use vpx_sys;

const DEFAULT_BITRATE: u32 = 512_u32;
pub const DEADLINE_USECS: u64 = 25_000_u64; // 25 ms is acceptable

#[repr(C)]
struct turnx_vpx_compander {
    bitrate: u32,

    cx_codec: vpx_sys::vpx_codec_ctx_t,
    dx_codec: vpx_sys::vpx_codec_ctx_t,

    cx_iface: *mut vpx_sys::vpx_codec_iface_t,
    dx_iface: *mut vpx_sys::vpx_codec_iface_t,

    frame: vpx_sys::vpx_codec_pts_t,

    cx_config: vpx_sys::vpx_codec_enc_cfg_t,
    dx_config: vpx_sys::vpx_codec_dec_cfg_t,
}

extern "C" {
    fn turnx_vpx_compander_create(w: u32, h: u32) -> *mut turnx_vpx_compander;
    fn turnx_vpx_compander_delete(codec: *mut turnx_vpx_compander);
}

#[derive(Clone)]
pub struct Compander {
    bitrate: u32,
    w: u32,
    h: u32,

    fifo: Vec<Vec<u8>>,
    codec: *mut turnx_vpx_compander,
}

impl Compander {
    pub fn new(w: u32, h: u32) -> Compander {
        Compander {
            bitrate: DEFAULT_BITRATE,
            w: w,
            h: h,
            fifo: vec![],
            codec: unsafe { turnx_vpx_compander_create(w, h) },
        }
    }

    pub unsafe fn set_bitrate(&mut self, rate: u32) {
        (*self.codec).cx_config.rc_target_bitrate = rate;
        let status = vpx_sys::vpx_codec_enc_config_set(
            &mut (*self.codec).cx_codec,
            &mut (*self.codec).cx_config,
        );
        if status == vpx_sys::VPX_CODEC_OK {
            self.bitrate = rate;
        }
        assert_eq!(status, vpx_sys::VPX_CODEC_OK);
    }

    pub fn get_bitrate(&self) -> u32 {
        self.bitrate
    }

    // push for reprocessing
    pub fn push(&mut self, frame: Vec<u8>) {
        self.fifo.push(frame)
    }

    // reprocess then pop
    pub unsafe fn pop(&mut self) -> Option<Vec<u8>> {
        let frame_to_reprocess = self.fifo.pop();

        match frame_to_reprocess {
            Some(frame_unwrapped) => {
                // Time to reprocess into the appropriate bitrate

                // DECODE
                vpx_sys::vpx_codec_decode(
                    &mut (*self.codec).dx_codec,
                    frame_unwrapped.as_ptr(),
                    frame_unwrapped.len() as u32,
                    std::ptr::null_mut(),
                    DEADLINE_USECS as i64,
                );

                let dx_image =
                    vpx_sys::vpx_codec_get_frame(&mut (*self.codec).dx_codec, std::ptr::null_mut());

                // Match it. Is it null? If so, encode.
                match Some(dx_image) {
                    Some(dx_valid_image) => {
                        // ENCODE
                        vpx_sys::vpx_codec_encode(
                            &mut (*self.codec).cx_codec,
                            &*dx_valid_image,
                            (*self.codec).frame,
                            1,
                            0,
                            DEADLINE_USECS,
                        );
                    }
                    None => {
                        // FLUSH ENCODE
                        vpx_sys::vpx_codec_encode(
                            &mut (*self.codec).cx_codec,
                            std::ptr::null(),
                            (*self.codec).frame,
                            1,
                            0,
                            DEADLINE_USECS,
                        );
                    }
                }
            }
            None => {
                // FLUSH ENCODE anyways
                vpx_sys::vpx_codec_encode(
                    &mut (*self.codec).cx_codec,
                    std::ptr::null(),
                    (*self.codec).frame,
                    1,
                    0,
                    DEADLINE_USECS,
                );
            }
        }

        let reprocessed_frame =
            vpx_sys::vpx_codec_get_cx_data(&mut (*self.codec).cx_codec, std::ptr::null_mut());

        match Some(reprocessed_frame) {
            Some(valid_reprocessed) => Some(Vec::from_raw_parts(
                (*valid_reprocessed).data.frame.buf as *mut u8,
                (*valid_reprocessed).data.frame.sz as usize,
                (*valid_reprocessed).data.frame.sz as usize,
            )),
            None => None,
        }
    }

    // Report on congestion
    pub fn len(&mut self) -> usize {
        self.fifo.len()
    }
}

impl Drop for Compander {
    fn drop(&mut self) {
        unsafe { turnx_vpx_compander_delete(self.codec) };
    }
}
