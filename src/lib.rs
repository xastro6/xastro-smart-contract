#![no_std]

extern crate alloc;
use alloc::format;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::instruction::{transfer, mint_to};
use borsh::{BorshDeserialize, BorshSerialize};

// Struct to store reward account data
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct RewardAccount {
    pub total_points: u32,
    pub rewards_claimed: u32,
    pub mint: Pubkey, // Mint address of "WAGUS" token
}

// Enum for different reward system instructions
#[derive(BorshSerialize, BorshDeserialize)]
pub enum RewardInstruction {
    Init,
    Earn { points: u32 },
    Claim { required_points: u32, amount: u64 },
    MintToken { amount: u64 },
}

// Entry point of the program
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Reward System Program Entry");
    msg!("Instruction data: {:?}", instruction_data);

    let accounts_iter = &mut accounts.iter();

    // Log all accounts for debugging
    for (i, acc) in accounts.iter().enumerate() {
        msg!("Account {}: {} is_signer: {}", i, acc.key, acc.is_signer);
    }

    // Extract signer as the first account
    let signer = next_account_info(accounts_iter)?;
    if !signer.is_signer {
        msg!("Missing required signature for signer: {}", signer.key);
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract and validate reward account as a PDA
    let reward_account_info = next_account_info(accounts_iter)?;
    let (reward_account_pda, reward_bump) = Pubkey::find_program_address(
        &[b"reward"],
        program_id
    );
    if reward_account_info.key != &reward_account_pda {
        msg!("Invalid reward account PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let user_token_account = next_account_info(accounts_iter)?;
    let vault_token_account = next_account_info(accounts_iter)?;
    let mint_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let instruction = RewardInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        RewardInstruction::Init => {
            // Define the exact size of RewardAccount
            const REWARD_ACCOUNT_SIZE: usize = 40; // 4 (u32) + 4 (u32) + 32 (Pubkey)

            // Check if the account already exists
            if reward_account_info.lamports() > 0 {
                if reward_account_info.owner == program_id {
                    // Check if the account data is valid by attempting deserialization
                    let is_valid = RewardAccount::try_from_slice(&reward_account_info.data.borrow()).is_ok();
                    if is_valid {
                        msg!("Account already initialized and valid");
                        return Ok(()); // Idempotent: account exists and is initialized, so skip
                    }

                    // If deserialization fails, the account data is invalid
                    msg!("Account exists but data is invalid. Reinitializing...");

                    // Overwrite the account by reallocating and initializing
                    let rent = Rent::get()?;
                    let space = REWARD_ACCOUNT_SIZE;
                    let rent_exemption_amount = rent.minimum_balance(space);

                    // Ensure the account has enough lamports for rent exemption
                    if reward_account_info.lamports() < rent_exemption_amount {
                        msg!("Account lamports insufficient for rent exemption: {} < {}", reward_account_info.lamports(), rent_exemption_amount);
                        let lamports_to_add = rent_exemption_amount - reward_account_info.lamports();
                        let transfer_ix = system_instruction::transfer(
                            signer.key,
                            reward_account_info.key,
                            lamports_to_add,
                        );
                        invoke(
                            &transfer_ix,
                            &[
                                signer.clone(),
                                reward_account_info.clone(),
                                system_program.clone(),
                            ],
                        )?;
                    }

                    // Reallocate the account to the correct size
                    reward_account_info.realloc(space, true)?;
                    msg!("Reallocated account to {} bytes", space);

                    // Initialize the reward account data
                    let mut data = reward_account_info.data.borrow_mut();
                    let reward_account = RewardAccount {
                        total_points: 0,
                        rewards_claimed: 0,
                        mint: *mint_account.key,
                    };
                    reward_account.serialize(&mut *data)?;
                    msg!("Rewrote invalid account data: {:?}", reward_account);
                } else {
                    msg!("Account exists but is not owned by the program: owner is {}", reward_account_info.owner);
                    return Err(ProgramError::InvalidAccountData);
                }
            } else {
                // If the account doesn’t exist, create and initialize it
                let rent = Rent::get()?;
                let space = REWARD_ACCOUNT_SIZE;
                let rent_exemption_amount = rent.minimum_balance(space);

                // Create the reward account PDA
                let create_account_ix = system_instruction::create_account(
                    signer.key,            // Payer
                    reward_account_info.key, // New account (PDA)
                    rent_exemption_amount, // Lamports for rent exemption
                    space as u64,         // Space
                    program_id,           // Owner (this program)
                );

                // Invoke with the program signing as the PDA
                invoke_signed(
                    &create_account_ix,
                    &[
                        signer.clone(),
                        reward_account_info.clone(),
                        system_program.clone(),
                    ],
                    &[&[b"reward", &[reward_bump]]],
                )?;

                // Initialize the reward account data
                let mut data = reward_account_info.data.borrow_mut();
                let reward_account = RewardAccount {
                    total_points: 0,
                    rewards_claimed: 0,
                    mint: *mint_account.key,
                };
                reward_account.serialize(&mut *data)?;
                msg!("Reward account initialized with data: {:?}", reward_account);
            }
        }

        RewardInstruction::Earn { points } => {
            if reward_account_info.owner != program_id {
                msg!("Account does not have the correct program id");
                return Err(ProgramError::IncorrectProgramId);
            }

            // Scope the immutable borrow to drop it before the mutable borrow
            let mut reward_account = {
                let raw_data = reward_account_info.data.borrow();
                msg!("Raw account data length: {}", raw_data.len());
                msg!("Raw account data: {:?}", raw_data);

                // Attempt to deserialize
                RewardAccount::try_from_slice(&raw_data)
                    .map_err(|e| {
                        msg!("Deserialization error: {:?}", e);
                        ProgramError::InvalidAccountData
                    })?
            };
            msg!("Deserialized reward account: {:?}", reward_account);

            reward_account.total_points = reward_account.total_points.checked_add(points)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            // Now it's safe to borrow mutably
            let mut data = reward_account_info.data.borrow_mut();
            reward_account.serialize(&mut *data)?;
            msg!("Earned {} points! Updated reward account: {:?}", points, reward_account);
        }

        RewardInstruction::Claim { required_points, amount } => {
            if reward_account_info.owner != program_id {
                msg!("Account does not have the correct program id");
                return Err(ProgramError::IncorrectProgramId);
            }

            let mut reward_account = RewardAccount::try_from_slice(&reward_account_info.data.borrow())?;

            if reward_account.total_points < required_points {
                msg!("Not enough points to claim reward!");
                return Err(ProgramError::InsufficientFunds);
            }

            reward_account.total_points -= required_points;
            reward_account.rewards_claimed += 1;
            let mut data = reward_account_info.data.borrow_mut();
            reward_account.serialize(&mut *data)?;

            let transfer_ix = transfer(
                token_program.key,
                vault_token_account.key,
                user_token_account.key,
                signer.key, 
                &[], 
                amount,
            )?;
            invoke(
                &transfer_ix,
                &[
                    vault_token_account.clone(),
                    user_token_account.clone(),
                    token_program.clone(),
                    signer.clone(),
                ],
            )?;

            msg!("Transferred {} WAGUS tokens as reward!", amount);
        }

        RewardInstruction::MintToken { amount } => {
            if reward_account_info.owner != program_id {
                msg!("Account does not have the correct program id");
                return Err(ProgramError::IncorrectProgramId);
            }

            msg!("Minting {} WAGUS tokens", amount);

            // Derive PDA to be used as mint authority
            let (mint_authority, bump_seed) = Pubkey::find_program_address(&[b"WAGUS"], program_id);

            if signer.key != &mint_authority {
                msg!("Invalid mint authority");
                return Err(ProgramError::InvalidSeeds);
            }

            let mint_ix = mint_to(
                token_program.key,
                mint_account.key,
                vault_token_account.key,
                &mint_authority,
                &[], 
                amount,
            )?;
            invoke_signed(
                &mint_ix,
                &[
                    mint_account.clone(),
                    vault_token_account.clone(),
                    token_program.clone(),
                    signer.clone(),
                ],
                &[&[b"WAGUS", &[bump_seed]]],
            )?;

            msg!("Minted {} WAGUS tokens!", amount);
        }
    }

    Ok(())
}
