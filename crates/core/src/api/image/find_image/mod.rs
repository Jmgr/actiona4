//! Client side of the out-of-process image matching extension.
//!
//! All OpenCV work lives in the `extension-opencv` binary; this module only
//! holds handles to images it has prepared and forwards search requests. The
//! wire types are re-exported so the rest of `core` keeps referring to
//! `find_image::{Match, FindImageProgress, ...}` as before.

pub mod client;
pub mod search_in;

pub use client::{OpenCVClient, ProgressHandler, ProgressRegistry, RemoteSource, RemoteTemplate};
pub use extension::protocols::opencv::{
    CaptureSpec, FindImageProgress, FindImageStage, FindImageTemplateOptions, Match,
};
pub use search_in::SearchIn;
