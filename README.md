# mmal-rs

Rust Wrappers for Multi-Media Abstraction Layer (MMAL) of Raspberry Pi

# What is `mmal-rs`?

`mmal-rs` provides Rust wrappers around MMAL's low level C-ctyle API:

* FFI wrappers (adapted from `mmal-sys` crate)
* High-level component handles, that hide complexity of C API and allow easy building complex MMAL applications in Rust

A number of example programs is provided.

# What is MMAL?

Multi-Media Abstraction Layer (MMAL) is a proprietary multimedia library that provides C API to Broadcom's VideoCore IV GPU, 
generally found on the Raspberry Pi.

Capabilities provided by MMAL include:

* Access to Raspberry Pi camera
* Video codecs

