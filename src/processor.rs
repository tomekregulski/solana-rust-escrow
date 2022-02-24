
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::invoke
};

use spl_token::state::Account as TokenAccount;

use crate::{error::EscrowError, instruction::EscrowInstruction, state::Escrow};

pub struct Processor;

impl Processor {
  pub fn processor(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let instruction = EscrowInstruction::unpack(instruction_data)?;

    match instruction {
      EscrowInstruction::InitEscrow { amount } => {
        msg!("Instruction: InitEscrow");
        Self::process_init_escrow(accounts, amount, program_id)
      }
    }
  }

  fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // needs to be mutable so we can take elements out of it.
        // The first account we expect - as defined in instruction.rs - is the escrow's initializer, i.e. INITIALIZER's main account. They need to be a signer which we check right away. It's just a boolean field on AccountInfo.
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        //  The temporary token account needs to be writable but there is no need to explicitly check this. The transaction will fail automatically should INITIALIZER not mark the account as writable.
        let temp_token_account = next_account_info(account_info_iter)?;

        let token_to_receive_account = next_account_info(account_info_iter)?;
        if *token_to_receive_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }
        
        let escrow_account = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(EscrowError::NotRentExempt.into());
        }

        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.try_borrow_data()?)?;
        if escrow_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // With Escrow struct instance created and and checked that it was not previously initialized, we now populate the struct's fields
        escrow_info.is_initialized = true;
        escrow_info.initializer_pubkey = *initializer.key;
        escrow_info.temp_token_account_pubkey = *temp_token_account.key;
        escrow_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
        escrow_info.expected_amount = amount;
        // Pack will call pack_into_slice
        Escrow::pack(escrow_info, &mut escrow_account.try_borrow_mut_data()?)?;
        // Create PDA by passing in an array of seeds, plus the program_id
        //
        // In our case the seeds can be static. There are cases such as in the Associated Token Account program where they aren't (because different users should own different associated token accounts). We just need 1 PDA that can own N temporary token accounts for different escrows occuring at any and possibly the same point in time.
        //
        // PDAs are public keys that are derived from the program_id and the seeds as well as having been pushed off the curve by the bump seed. Hence, Program Derived Addresses do not lie on the ed25519 curve and therefore have no private key associated with them.
        //
        // A PDA is just a random array of bytes with the only defining feature being that they are not on that curve. That said, they can still be used as normal addresses most of the time. 
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

        // Invoke CPI to transfer the (user space) ownership of the temporary token account to the PDA. 

        // First, create the token_program account. The program being called through a CPI must be ingcluded in the 2nd argument as an account. 
        let token_program = next_account_info(account_info_iter)?;
        // set_authority is a builder function that creates the instruction for the token program
        //
        // We pass in: 
        //  the token program id, 
        //  then the account whose authority we'd like to change, 
        //  the account that's the new authority (in our case the PDA), 
        //  the type of authority change (there are different authority types for token accounts, we care about changing the main authority), 
        //  the current account authority (INITIALIZER -> initializer.key), 
        //  and finally the public keys signing the CPI.
        //
        // The conept being used here is called Signature Extension, in short:
        //
        //  When including a signed account in a program call, in all CPIs including that account made by that program inside the current instruction, the account will also be signed, i.e. the signature is extended to the CPIs.
        //
        //  In our case this means that because INITIALIZER signed the InitEscrow transaction, the program can make the token program set_authority CPI and include their pubkey as a signer pubkey. This is necessary because changing a token account's authority should of course require the approval of the current authority.
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        )?;

        // Note that before making a CPI, we should add another check that the token_program is truly the account of the token program. Otherwise, we might be calling a rogue program. If you're using the spl-token crate above version 3.1.1 (which I do in this guide), you don't have to do this if you use their instruction builder functions. They do it for you.

        msg!("Calling the token program to transfer token account ownership...");
        invoke(
            &owner_change_ix,
            &[
                temp_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }
}