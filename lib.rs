#![no_std]

extern crate alloc;
use alloc::format;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use spl_token::instruction::transfer;
use core::mem;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct RewardAccount {
    pub total_points: u32,
    pub rewards_claimed: u32,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum RewardInstruction {
    Init,
    Earn { points: u32 },
    Claim,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Reward System Program Entry");

    let accounts_iter = &mut accounts.iter();
    let reward_account_info = next_account_info(accounts_iter)?;
    let user_token_account = next_account_info(accounts_iter)?;
    let vault_token_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let signer = next_account_info(accounts_iter)?;

    if !signer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if reward_account_info.owner != program_id {
        msg!("Account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    let instruction = RewardInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        RewardInstruction::Init => {
            if !reward_account_info.data_is_empty() {
                msg!("Account already initialized");
                return Err(ProgramError::AccountAlreadyInitialized);
            }
            let rent = Rent::get()?;
            let required_lamports = rent.minimum_balance(mem::size_of::<RewardAccount>());
            if **reward_account_info.lamports.borrow() < required_lamports {
                msg!("Insufficient lamports for rent");
                return Err(ProgramError::InsufficientFunds);
            }
            let reward_account = RewardAccount::default();
            reward_account.serialize(&mut &mut reward_account_info.data.borrow_mut()[..])?;
            msg!("Reward account initialized!");
        }
        RewardInstruction::Earn { points } => {
            let mut reward_account = RewardAccount::try_from_slice(&reward_account_info.data.borrow())?;
            reward_account.total_points = reward_account.total_points.checked_add(points)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            reward_account.serialize(&mut &mut reward_account_info.data.borrow_mut()[..])?;
            msg!("Earned {} points!", points);
        }
        RewardInstruction::Claim => {
            let mut reward_account = RewardAccount::try_from_slice(&reward_account_info.data.borrow())?;
            if reward_account.total_points < 100 {
                msg!("Not enough points to claim a reward.");
                return Err(ProgramError::InsufficientFunds);
            }
            reward_account.total_points -= 100;
            reward_account.rewards_claimed = reward_account.rewards_claimed.checked_add(1)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            let (vault_authority, _bump_seed) = Pubkey::find_program_address(&[b"vault"], program_id);
            let transfer_ix = transfer(
                token_program.key,
                vault_token_account.key,
                user_token_account.key,
                &vault_authority,
                &[],
                1_000_000,
            ).map_err(|_| ProgramError::Custom(1001))?;  // Replace `1001` with a suitable error code

            invoke(
                &transfer_ix,
                &[
                    token_program.clone(),
                    vault_token_account.clone(),
                    user_token_account.clone(),
                ],
            )?;
            msg!("Transferred 1 WAGUS!");
            reward_account.serialize(&mut &mut reward_account_info.data.borrow_mut()[..])?;
            msg!("Successfully claimed a reward!");
        }
    }

    let reward_account = RewardAccount::try_from_slice(&reward_account_info.data.borrow())?;
    msg!(
        "Total points: {}, Rewards claimed: {}",
        reward_account.total_points,
        reward_account.rewards_claimed
    );

    Ok(())
}
