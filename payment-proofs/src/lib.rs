#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

mod shared_key_derivations;
mod out_proof;
pub use out_proof::OutProof;

#[cfg(test)]
mod tests;
