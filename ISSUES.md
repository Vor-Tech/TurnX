# TurnX Issues/TODOs
## General
- Implement Opus audio.

## Elixir
- SRTP needs to actually be implemented.
- Supervision trees need to restart the Rust/C++ port if and when it fails.
- Test Driven Development cases for SRTP + H264.

## Rust
- Implement Erlang Port for Elixir interface

## C++
- (DONE) Make `turnx_h264_cxxcalls_smoketest`'s static assertion failure messages more verbose.
- (Almost done but **needs testing**) Complete the OpenH264 [adaptive bitrate streamer](https://en.wikipedia.org/wiki/Adaptive_bitrate_streaming) actually.
- Seems like more documentation is needed.
