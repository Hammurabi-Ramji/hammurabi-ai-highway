// src/lib.rs — Library entry point

pub mod config;
// TODO(crypto): key_manager.rs is aspirational scaffolding that does not compile
// against the pinned crate versions (argon2 0.5 has no `hash_raw`/`Config`, the
// `SecureStorage` async trait is not object-safe behind `Arc<dyn>`, `rand::rng()`
// is a rand 0.9 API while 0.8 is pinned, `futures` is not a dependency, and it
// calls non-existent methods). It is unused elsewhere. Deferred until rewritten
// and verified against the actual APIs.
// pub mod crypto;
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
