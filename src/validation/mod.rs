// src/validation/mod.rs — Calldata validation module

mod calldata_verifier;

// Standalone calldata verifier. Not yet wired into the live pipeline (the
// on-chain path uses `venice::Calldata`); exercised directly by tests via the
// library target, hence unused from the binary's perspective.
#[allow(unused_imports)]
pub use calldata_verifier::{Calldata, CalldataVerificationResult, CalldataVerifier};
