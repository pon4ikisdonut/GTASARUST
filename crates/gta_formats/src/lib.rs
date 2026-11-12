//! Clean-room, bounds-checked parsers for original Grand Theft Auto: San Andreas
//! binary and text resource formats. Zero `unsafe` code is used in this crate.

#![forbid(unsafe_code)]

pub mod audio_cfg;
pub mod col;
pub mod cut;
pub mod dat;
pub mod error;
pub mod gxt;
pub mod ide;
pub mod ifp;
pub mod img;
pub mod ipl;
pub mod paths;
pub mod reader;
pub mod rw;
pub mod scm;
pub mod scm_opcodes;

pub use error::FormatError;
pub use img::{ImgArchive, ImgEntry};
