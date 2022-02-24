use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum EscrowError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Expected Amount Mismatch
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    /// Amount Overflow
    #[error("Amount Overflow")]
    AmountOverflow,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// Let's stop for a moment to understand what is happening here. We are implementing a generic trait, specifically the From (opens new window)trait which the ? operator wants to use. To implement this trait we have to implement the from function which carries out the conversion. The ProgramError enum provides the Custom variant that allows us to convert from our program's EscrowError to a ProgramError.

/// The reason we do this conversion in the first place is that the entrypoint returns a Result of either nothing or a ProgramError.
