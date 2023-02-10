#![deny(missing_docs)]
//! Utilities for Exponential-Golomb coding.

mod decoder;
mod encoder;

pub use self::{decoder::ExpGolombDecoder, encoder::ExpGolombEncoder};
