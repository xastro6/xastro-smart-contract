use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct RewardAccount {
    pub total_points: u32,
    pub rewards_claimed: u32,
}

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8], // now used to define action
) -> ProgramResult {
    msg!("Reward System Program Entry");   

    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    if account.owner != program_id {
        msg!("Account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut reward_account = RewardAccount::try_from_slice(&account.data.borrow())?;

    // Handle instruction data
    match instruction_data {
        b"earn" => {
            reward_account.total_points += 10;
            msg!("Earned 10 points!");
        }
        b"claim" => {
            if reward_account.total_points >= 100 {
                reward_account.total_points -= 100;
                reward_account.rewards_claimed += 1;
                msg!("Successfully claimed a reward!");
            } else {
                msg!("Not enough points to claim a reward.");
                return Err(ProgramError::InsufficientFunds);
            }
        }
        _ => {
            msg!("Invalid instruction");
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    reward_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    msg!(
        "Total points: {}, Rewards claimed: {}",
        reward_account.total_points,
        reward_account.rewards_claimed
    );

    Ok(())
}