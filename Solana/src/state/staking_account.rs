use anchor_lang::prelude::*;
use serde::{Serialize, Deserialize};

use crate::{
    constants::constants, errors::ErrorCode, state::GlobalConfig
};

use super::UserStakedAccount;

#[account]
#[derive(Debug, Serialize, Deserialize)]
pub struct StakingAccount {
    pub user: Pubkey,                         // Owner of the staking account
    pub claim_airdrop: bool,                  // Airdrop status
    pub total_staked: u64,                    // Total staked amount by the user
    pub total_referral_staked: u64,           // Total staked amount by the referrals
    pub daily_referral_rewards: u64,          // Referral rewards for daily claim
    pub user_staked_counter: u64,             // Counter to track UserStakedAccounts
    pub last_referral_rewards_claimed: i64,   // Last referral rewards claimed timestamp
    pub referrer: Pubkey,                     // Referrer of the user
    pub referral_history: Vec<Pubkey>,        // Referral history addresses
}

impl StakingAccount {
    pub const LEN: usize = 8 + std::mem::size_of::<StakingAccount>() + 8 + (32 * constants::MAX_REFERRAL);

    pub fn is_initialized(&self) -> bool {
        self.user != Pubkey::default()
    }

    pub fn calculate_referral_rewards(
        &self,
        global_config: &Account<GlobalConfig>,
        now: i64,
    ) -> Result<u64> {
        let mut rewards: u128 = 0;
        let last_claimed = self.last_referral_rewards_claimed;
        let principal = self.daily_referral_rewards as u128;
        let apy_history = global_config.apy_history.clone();
        let previous_apy = apy_history.first().unwrap().apy_bps as u128;
        let current_apy_start = apy_history.first().unwrap().timestamp;
        let current_apy = global_config.current_apy_bps as u128;

        let mut last_claimed_day = last_claimed - (last_claimed % constants::CLAIM_PERIOD_SECONDS);
        let current_apy_start_day = current_apy_start - (current_apy_start % constants::CLAIM_PERIOD_SECONDS);

        if last_claimed_day < current_apy_start_day {
            let interval = (current_apy_start_day - last_claimed_day) / constants::CLAIM_PERIOD_SECONDS;

            let rewards_by_day = principal
                .checked_mul(previous_apy)
                .ok_or(ErrorCode::Overflow)?
                .checked_mul(constants::CLAIM_PERIOD_SECONDS as u128)
                .ok_or(ErrorCode::Overflow)?
                .checked_div(365 * 24 * 60 * 60 * 10_000) // Adjust based on staking period
                .ok_or(ErrorCode::Overflow)?;

            rewards += rewards_by_day * interval as u128;

            last_claimed_day = current_apy_start_day;
        }

        let today = now - (now % constants::CLAIM_PERIOD_SECONDS);
        let interval = (today - last_claimed_day) / constants::CLAIM_PERIOD_SECONDS;

        let rewards_by_day = principal
            .checked_mul(current_apy)
            .ok_or(ErrorCode::Overflow)?
            .checked_mul(constants::CLAIM_PERIOD_SECONDS as u128)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(365 * 24 * 60 * 60 * 10_000) // Adjust based on staking period
            .ok_or(ErrorCode::Overflow)?;

        rewards += rewards_by_day * interval as u128;

        msg!("Total rewards: {:?}", rewards);

        Ok(u64::try_from(rewards).map_err(|_| ErrorCode::Overflow)?)
    }

    pub fn calculate_all_rewards(
        &self,
        remaining_accounts: &[AccountInfo],
        global_config: &Account<GlobalConfig>,
        now: i64
    ) -> Result<u64> {
        let mut total_rewards = 0_u64;
    
        for account_info in remaining_accounts.iter() {
            let user_staked_account_data: UserStakedAccount = {
                let user_staked_account = account_info.try_borrow_data()?;
                UserStakedAccount::try_deserialize(&mut &user_staked_account[..])
                    .map_err(|_| ErrorCode::DeserializationError)?
            };
    
            if user_staked_account_data.user.key() != self.user.key() {
                return Err(ErrorCode::InvalidUser.into());
            }
    
            if user_staked_account_data.staked_timestamp + constants::ONE_YEAR_PERIOD < now {
                continue;
            }
    
            let last_claim_time = user_staked_account_data.last_claimed_timestamp;
    
            if now - last_claim_time < constants::CLAIM_PERIOD_SECONDS {
                msg!(
                    "Claim too soon for account: {:?}. You may claim at {:?}",
                    account_info.key(),
                    last_claim_time + constants::CLAIM_PERIOD_SECONDS
                );
                continue;
            }
    
            let reward_lamports_u64 = user_staked_account_data.calculate_rewards(global_config, now)?;
    
            if reward_lamports_u64 == 0 {
                msg!("No rewards available to claim for account: {:?}", account_info.key());
                continue;
            }
    
            total_rewards += reward_lamports_u64;
        }
    
        Ok(total_rewards)
    }
}
