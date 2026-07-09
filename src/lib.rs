// src/lib.rs — Library entry point

pub mod config;
pub mod crypto;
pub mod error;
pub mod metrics;
pub mod oneshot;
pub mod pipeline;
pub mod security;
pub mod sovereign;
pub mod validation;
pub mod venice;
pub mod x402;

pub use error::PipelineError;
