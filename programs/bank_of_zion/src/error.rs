#![allow(missing_docs)]

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::{ProgramError, PrintProgramError}, msg};
use thiserror::Error;
use num_traits::FromPrimitive;

/// Errors that may be returned by the Ban of Zion program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum ZionError {
    
    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Incorrect Program ID")]
    IncorrectTokenProgramId,

    #[error("Expected a Token account")]
    ExpectedTokenAccount,

    #[error("Expected a mint account")]
    ExpectedMint,

    #[error("The pool is already initialized")]
    PoolAlreadyInitialized,

    #[error("The pool is not initialized")]
    PoolNotInitialized,

    #[error("The owner of the Swap State is incorrect")]
    SwapStateWrongOwner,

    #[error("Swap Authority is invalid")]
    InvalidSwapAuthority,

    #[error("Token program ID is invalid")]
    InvalidTokenProgramKey,

    #[error("System program ID is invalid")]
    InvalidSystemProgramKey,

    #[error("The account is not owned by the token program")]
    NotOwnedByTokenProgram,

    #[error("Mint's in the pool can't be identical")]
    IdenticalMints,

    #[error("Supply must be 0")]
    InvalidSupply,

    #[error("Invalid token amount")]
    InvalidTokenAmount,

    #[error("Zero tokens provided")]
    ZeroTokens,

    #[error("Authority is invalid")]
    InvalidAuthority,
    #[error("Owner is invalid")]
    InvalidOwner,
    #[error("The SwapState is invalid")]
    InvalidSwapState,
    
    #[error("The Mint isn't initialized")]
    MintNotInitialized,
    #[error("The Swap State is initialized")]
    SwapStateInitialized,

    #[error("The Mint isn't valid")]
    InvalidMint,
    #[error("The Vault isn't valid")]
    InvalidVault,
    #[error("The Fee Vault isn't valid")]
    InvalidFeeVault,
    #[error("The Swap Mint isn't valid")]
    InvalidSwapMint,
    #[error("The Oracle isn't valid")]
    InvalidOracle,
    #[error("For this CTF decimals need to be the same")]
    DecimalsDifferent,
    #[error("Must be admin to execute this instruction")]
    MustBeAdmin,
    #[error("Must be signer")]
    InvalidSigner,
    #[error("Insufficient swap tokens")]
    InsufficientSwapTokens,

}

impl From<ZionError> for ProgramError {
    fn from(e: ZionError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for ZionError {
    fn type_of() -> &'static str {
        "Swap Error"
    }
}

impl PrintProgramError for ZionError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            ZionError::InvalidInstruction => {
                msg!("Invalid instruction")
            },
            ZionError::IncorrectTokenProgramId => {
                msg!("Error: Incorrect Program ID")
            }
            ZionError::ExpectedTokenAccount => {
                msg!("Error: Expected a Token account")
            }
            
            ZionError::ExpectedMint => {
                msg!("Error: Expected a mint account")
            }
            
            ZionError::PoolAlreadyInitialized => {
                msg!("Error: The pool is already initialized")
            }
            
            ZionError::PoolNotInitialized => {
                msg!("Error: The pool is not initialized")
            }
            ZionError::SwapStateWrongOwner => {
                msg!("Error: The owner of the Swap State is incorrect")
            }
            
            ZionError::InvalidSwapAuthority => {
                msg!("Error: The Swap Authority is invalid")
            }
            ZionError::InvalidAuthority => {
                msg!("Error: The Authority is invalid")
            }
            ZionError::InvalidOwner => {
                msg!("Error: The Owner is invalid")
            }
            
            ZionError::InvalidSwapState => {
                msg!("Error: The SwapState is invalid")
            }
            
            ZionError::InvalidTokenProgramKey => {
                msg!("Error: The Token program ID is invalid")
            }
            ZionError::InvalidSystemProgramKey => {
                msg!("Error: The System program ID is invalid")
            }

            ZionError::NotOwnedByTokenProgram => {
                msg!("Error: The account is not owned by the token program")
            }

            ZionError::IdenticalMints => {
                msg!("Error: Mint's in the pool can't be identical")
            }
            ZionError::InvalidSupply => {
                msg!("Error: Supply must be 0")
            }
            ZionError::InvalidTokenAmount => {
                msg!("Invalid token amount")
            }
            ZionError::ZeroTokens => {
                msg!("Zero tokens provided")
            }
            ZionError::MintNotInitialized => {
                msg!("Mint not initialized")
            }
            ZionError::SwapStateInitialized => {
                msg!("Swap State is initialized")
            }
            ZionError::InvalidMint => {
                msg!("Mint is invalid")
            }
            ZionError::InvalidVault => {
                msg!("Vault is invalid")
            }
            ZionError::InvalidFeeVault => {
                msg!("Fee Vault is invalid")
            }
            ZionError::InvalidSwapMint => {
                msg!("Swap Mint is invalid")
            }
            ZionError::InvalidOracle => {
                msg!("Oracle is invalid")
            }
            ZionError::DecimalsDifferent=> {
                msg!("For this CTF decimals need to be the same")
            }
            ZionError::MustBeAdmin=> {
                msg!("Must be admin to execute this instruction")
            }
            ZionError::InvalidSigner=> {
                msg!("Must be signer")
            }
            ZionError::InsufficientSwapTokens=> {
                msg!("Insufficient swap tokens")
            }


        }
    }
}