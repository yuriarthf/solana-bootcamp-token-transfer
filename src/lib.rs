use borsh::BorshDeserialize;
use solana_program::{
    entrypoint::ProgramResult,
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    pubkey::Pubkey,
    program_error::ProgramError,
    program_pack::Pack,
    msg,
    program::invoke_signed
};
use spl_token::{
    instruction::transfer_checked,
    state::{Account, Mint}
};

mod instruction;

use instruction::InstructionArgs;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {

    let account_iter = &mut accounts.iter();
    
    let source_info = next_account_info(account_iter)?;
    let mint_info = next_account_info(account_iter)?;
    let destination_info = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;
    let token_program_info = next_account_info(account_iter)?;

    let (expected_authority, bump_seed) = Pubkey::find_program_address(&[b"authority"], program_id);

    if expected_authority != *authority_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let source_account = Account::unpack(&source_info.try_borrow_data()?)?;
    let amount = InstructionArgs::try_from_slice(instruction_data)?.amount;
    if source_account.amount < amount {
        return Err(ProgramError::InsufficientFunds);
    }

    let mint = Mint::unpack(&mint_info.try_borrow_data()?)?;
    let decimals = mint.decimals;

    msg!("Attempting to transfer {} tokens", amount);

    invoke_signed(
        &transfer_checked(
            token_program_info.key,
            source_info.key,
            mint_info.key,
            destination_info.key,
            authority_info.key,
            &[],
            amount,
            decimals
        ).unwrap(),
        &[
            source_info.clone(),
            mint_info.clone(),
            destination_info.clone(),
            authority_info.clone()
        ],
        &[&[&[bump_seed]]]
    )?;

    Ok(())
}
