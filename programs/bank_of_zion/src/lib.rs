#![deny(missing_docs)]
#![cfg_attr(not(test), forbid(unsafe_code))]

//! An ERC20-like Token program for the Solana blockchain

///Defines all of the program errors
pub mod error;
///Contains all of the programs instructions
pub mod instructions;
///Processes all of the programs instructions
pub mod processor;
///Contains all of the programs states
pub mod state;
///contains all of the cross program invocations
pub mod cpi;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;


solana_program::declare_id!("DYPr7THq2b8chVndfn6TKuEUxJdKYhfRNtzkM2Lzzm8s");