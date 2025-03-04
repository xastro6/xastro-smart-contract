#![no_std] // Disable the standard library to create a no-std program for Solana

// External crate dependencies
extern crate alloc;
use alloc::format;

use solana_program::{
    account_info::{next_account_info, AccountInfo},  // For accessing account info
    entrypoint,                                       // Entry point for the Solana program
    entrypoint::ProgramResult,                        // Return type for program functions
    msg,                                              // Macro for logging messages
    program::invoke,                                  // Invoke another instruction
    program_error::ProgramError,                      // Error types for the program
    pubkey::Pubkey,                                   // Public key type
    rent::Rent,                                       // Rent system used for accounts
    sysvar::Sysvar,                                   // Access to system variables (like rent)
};
use spl_token::instruction::transfer;                 // Transfer instruction for the SPL Token program
use core::mem;                                       // For memory-related operations
use borsh::{BorshDeserialize, BorshSerialize};        // For (de)serialization of data structures

// Struct to store reward account data
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct RewardAccount {
    pub total_points: u32,        // Total reward points accumulated
    pub rewards_claimed: u32,     // Number of rewards claimed
}

// Enum for different reward system instructions
#[derive(BorshSerialize, BorshDeserialize)]
pub enum RewardInstruction {
    Init,                         // Initialize a new reward account
    Earn { points: u32 },         // Earn points, specifying how many
    Claim,                        // Claim a reward (spend points)
}

// Entry point of the program
entrypoint!(process_instruction);

// Main function to process the instructions
pub fn process_instruction(
    program_id: &Pubkey,               // The public key of the program being executed
    accounts: &[AccountInfo],          // List of accounts involved in the instruction
    instruction_data: &[u8],           // Data passed with the instruction (e.g., action type)
) -> ProgramResult {
    // Log the entry point of the reward system program
    msg!("Reward System Program Entry");

    // Get account info: reward account, user token account, vault, token program, and signer
    let accounts_iter = &mut accounts.iter();
    let reward_account_info = next_account_info(accounts_iter)?;
    let user_token_account = next_account_info(accounts_iter)?;
    let vault_token_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let signer = next_account_info(accounts_iter)?;

    // Ensure that the signer has signed the transaction
    if !signer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the reward account is owned by the correct program
    if reward_account_info.owner != program_id {
        msg!("Account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize instruction data into the RewardInstruction enum
    let instruction = RewardInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Match on the instruction to perform the correct action
    match instruction {
        // Case for initializing a reward account
        RewardInstruction::Init => {
            // Check if the account is already initialized
            if !reward_account_info.data_is_empty() {
                msg!("Account already initialized");
                return Err(ProgramError::AccountAlreadyInitialized);
            }

            // Ensure there are enough lamports for rent (storage fees)
            let rent = Rent::get()?;
            let required_lamports = rent.minimum_balance(mem::size_of::<RewardAccount>());
            if **reward_account_info.lamports.borrow() < required_lamports {
                msg!("Insufficient lamports for rent");
                return Err(ProgramError::InsufficientFunds);
            }

            // Initialize the reward account with default values
            let reward_account = RewardAccount::default();
            reward_account.serialize(&mut &mut reward_account_info.data.borrow_mut()[..])?;
            msg!("Reward account initialized!");
        }

        // Case for earning points
        RewardInstruction::Earn { points } => {
            // Deserialize the reward account data
            let mut reward_account = RewardAccount::try_from_slice(&reward_account_info.data.borrow())?;

            // Add points to the total and handle overflow
            reward_account.total_points = reward_account.total_points.checked_add(points)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            
            // Save the updated reward account back to the account
            reward_account.serialize(&mut &mut reward_account_info.data.borrow_mut()[..])?;
            msg!("Earned {} points!", points);
        }

        // Case for claiming a reward
        RewardInstruction::Claim => {
            // Deserial
