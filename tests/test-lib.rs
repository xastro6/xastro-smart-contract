pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Reward System Program Entry");

    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    if account.owner != program_id {
        msg!("Account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut reward_account = RewardAccount::try_from_slice(&account.data.borrow())?;

    // Process different instructions
    match instruction_data {
        b"earn" => {
            // Example: Add 10 points for "earn" action
            reward_account.total_points += 10;
            msg!("Earned points: 10");
        }
        b"claim" => {
            // Example: Claim reward
            if reward_account.total_points >= 100 {
                reward_account.total_points -= 100;
                reward_account.rewards_claimed += 1;
                msg!("Claimed reward: 100 points spent");
            } else {
                msg!("Not enough points to claim reward");
                return Err(ProgramError::InsufficientFunds);
            }
        }
        _ => {
            msg!("Unknown instruction");
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