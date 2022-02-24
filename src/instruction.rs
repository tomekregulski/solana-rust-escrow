// inside instruction.rs
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

use crate::error::EscrowError::InvalidInstruction;

pub enum EscrowInstruction {
  /// Starts the trade by creating and populating an escrow account and transferring ownership of the given temp token account to the PDA
  ///
  /// Although instruction.rs does not touch accounts, it is helpful to define which accounts you expect here so all the required calling info is in one place and easy to find for others.
  /// Accounts expected:
  ///
  /// 0. `[signer]` The account of the person initializing the escrow
  /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
  /// 2. `[]` The initializer's token account for the token they will receive should the trade go through
  /// 3. `[writable]` The escrow account, it will hold all necessary info about the trade.
  /// 4. `[]` The rent sysvar
  /// 5. `[]` The token program
  /// 
  /// Note re: "writeable" - If the caller does not mark the account writable in their calling code but the program attempts to write to it, the transaction will fail.
  /// 
  /// Further explanation:
  /// 0. Signer: We need Account 0 and specifically Account 0 as a signer because transferring the ownership of the temporary account requires the INITIALIZER'S signature.
  /// 1. Account 1 is the temp token X account which needs to be writable. This is because changing token account ownership is a user space change which means the data field of the account will be changed
  /// 2. Account 2 is INITIALIZER'S token Y account. While it will be written to eventually, it won't happen in this transaction which is why we can leave the brackets empty (implying read-only)
  /// 3. Account 3 is the escrow account which also needs to be writable because the program will write the escrow information into it
  /// 4. Account 4 is explained further in PROCESSOR
  /// 5. Account 5 is the account of the token program itself, which is explained fursther in PROCESSOR
  InitEscrow {
    /// The amount party A expects to receive of token Y
    amount: u64
  }
}

  /// Below:
  /// 1. unpack expects a reference (opens new window)to a slice of u8. 
  /// 2. It looks at the first byte (=tag) to determine how to decode (using match (opens new window)) the rest (=rest) of the slice. 
  /// 3. unpack_amount decodes the rest to get a u64 representing the amount. 
  /// Summary: choose which instruction to build and build/return that instruction.

impl EscrowInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitEscrow {
                amount: Self::unpack_amount(rest)?,
            },
            1 => Self::Exchange {
                amount: Self::unpack_amount(rest)?
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}

// LOOK INTO FRONTRUNNING

/// Accepts a trade
///
///
/// Accounts expected:
///
/// 0. `[signer]` The account of the person taking the trade
/// 1. `[writable]` The taker's token account for the token they send 
/// 2. `[writable]` The taker's token account for the token they will receive should the trade go through
/// 3. `[writable]` The PDA's temp token account to get tokens from and eventually close
/// 4. `[writable]` The initializer's main account to send their rent fees to
/// 5. `[writable]` The initializer's token account that will receive tokens
/// 6. `[writable]` The escrow account holding the escrow info
/// 7. `[]` The token program
/// 8. `[]` The PDA account
Exchange {
    /// the amount the taker expects to be paid in the other token, as a u64 because that's the max possible supply of a token
    amount: u64,
}