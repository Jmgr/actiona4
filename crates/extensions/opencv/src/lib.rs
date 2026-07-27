//! Out-of-process image template matching.
//!
//! This extension owns every use of OpenCV in the project. The main binary
//! talks to it over the `extension` IPC framework, which keeps `actiona-run`
//! free of the OpenCV toolchain and isolates it from crashes inside the
//! matching code.

pub mod capture;
pub mod find_image;
pub mod handler;
