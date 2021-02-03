// ABR: Adaptive Bitrate Encoding/Decoding
#[repr(C)]
pub struct turnx_h264_calls_frame {
    is_encoded: bool,
    n_items: i32,
    data: [*mut u8; 3],
    buf: *mut u8,
}

extern "C" {
    pub fn turnx_h264_cxxcalls_start(w: u16, h: u16);
    pub fn turnx_h264_cxxcalls_stop();
    pub fn turnx_h264_cxxcalls_push(enc_frame: *const turnx_h264_calls_frame);
    pub fn turnx_h264_cxxcalls_pop() -> *const turnx_h264_calls_frame;
    pub fn turnx_h264_cxxcalls_size() -> usize;
}
