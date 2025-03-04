pub fn process_instruction(
    program_id: &Pubkey,          // The public key of the program that is executing the instruction
    accounts: &[AccountInfo],     // The accounts involved in this instruction call
    instruction_data: &[u8],      // Data that specifies what action to take (earn, claim, etc.)
) -> ProgramResult {
    // Log the entry point of the reward system program
    msg!("Reward System Program Entry");

    // Create an iterator over the accounts and get the first account
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    // Ensure the account's owner matches the program's public key
    if account.owner != program_id {
        // Log the error if the account's owner is incorrect
        msg!("Account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize the data from the account to load the RewardAccount structure
    let mut reward_account = RewardAccount::try_from_slice(&account.data.borrow())?;

    // Match the instruction data to decide which action to take
    match instruction_data {
        // If instruction is "earn", add 10 points to the reward account
        b"earn" => {
            reward_account.total_points += 10; // Add 10 points to total points
            msg!("Earned points: 10"); // Log the earned points
        }
        // If instruction is "claim", attempt to claim a reward
        b"claim" => {
            // Check if the account has enough points to claim a reward
            if reward_account.total_points >= 100 {
                reward_account.total_points -= 100; // Deduct 100 points for the claim
                reward_account.rewards_claimed += 1; // Increment the rewards claimed counter
                msg!("Claimed reward: 100 points spent"); // Log the claim action
            } else {
                // Log an error and return if there aren't enough points
                msg!("Not enough points to claim reward");
                return Err(ProgramError::InsufficientFunds);
            }
        }
        // If instruction is neither "earn" nor "claim", return an error for invalid data
        _ => {
            msg!("Unknown instruction");
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    // Serialize the updated reward account back into the account's data
    reward_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    // Log the updated state of the reward account
    msg!(
        "Total points: {}, Rewards claimed: {}",
        reward_account.total_points,
        reward_account.rewards_claimed
    );

    // Return Ok indicating the instruction processed successfully
    Ok(())
}
