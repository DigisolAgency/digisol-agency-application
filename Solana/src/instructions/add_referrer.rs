use crate::{
    constants, errors::ErrorCode, state::{GlobalConfig, StakingAccount}
};

use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(new_referrer: Pubkey)]
pub struct AddReferrer<'info> {
    #[account(
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [b"staking", user_key.key().as_ref()],
        bump
    )]
    pub staking_account: Account<'info, StakingAccount>,

    #[account(
        seeds = [b"staking", new_referrer.key().as_ref()],
        bump
    )]
    pub referrer_staking_account: Account<'info, StakingAccount>,

    /// CHECK: user account
    pub user_key: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
}

pub fn add_referrer(ctx: Context<AddReferrer>, new_referrer: Pubkey) -> Result<()> {
    let global_config = &ctx.accounts.global_config;
    let staking_account = &mut ctx.accounts.staking_account;
    let referrer_account = &ctx.accounts.referrer_staking_account;
    let signer_key = ctx.accounts.user.key();
    let user_key = ctx.accounts.user_key.key();

    require!(new_referrer != Pubkey::default(), ErrorCode::InvalidReferrer);
    require!(new_referrer != staking_account.user, ErrorCode::InvalidReferrer);

    if signer_key != user_key {
        require!(signer_key == global_config.admin, ErrorCode::OnlyAdmin);
    } else {
        require!(staking_account.referrer == Pubkey::default(), ErrorCode::AlreadyHaveReferrer);
    }

    let staked_amount = staking_account.total_staked;

    let remaining_accounts = ctx.remaining_accounts;

    require!(remaining_accounts.len() > 0, ErrorCode::ShouldProvideReferrer);

    if remaining_accounts.len() > global_config.lvl_percentages.len() {
        return Err(ErrorCode::OnlyTenReferralsLevel.into());
    }

    apply_referral_rewards_for_all_referrer(
        remaining_accounts,
        global_config,
        staked_amount,
        referrer_account.key(),
        staking_account.key()
    )?;

    staking_account.referrer = referrer_account.key();

    msg!("Referrer added successfully {}", new_referrer);

    Ok(())
}

fn apply_referral_rewards_for_all_referrer(
    remaining_accounts: &[AccountInfo],
    global_config: &Account<GlobalConfig>,
    stake_amount: u64,
    first_referrer: Pubkey,
    referral: Pubkey
) -> Result<()> {
    let mut next_referrer = first_referrer;

    for (index, account_info) in remaining_accounts.iter().enumerate() {
        if next_referrer != account_info.key() {
            return Err(ErrorCode::ReferrerMismatch.into());
        }

        let referrer_account = &mut account_info.try_borrow_mut_data()?[..];

        let mut referrer_data: StakingAccount = StakingAccount::try_deserialize(&mut &referrer_account[..])
            .map_err(|_| ErrorCode::DeserializationError)?;

        next_referrer = referrer_data.referrer;

        if index == 0 {
            if referrer_data.referral_history.len() < constants::MAX_REFERRAL {
                referrer_data.referral_history.push(referral);
            }
        }

        let commission_percentage = global_config.lvl_percentages.get(index).copied().unwrap_or(0);

        let commission = stake_amount
            .checked_mul(commission_percentage as u64)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(100)
            .ok_or(ErrorCode::Overflow)?;

        if commission > 0 {
            if referrer_data.total_referral_staked == 0 {
                referrer_data.last_referral_rewards_claimed = Clock::get()?.unix_timestamp;
            }

            referrer_data.total_referral_staked = referrer_data
                .total_referral_staked
                .checked_add(stake_amount)
                .ok_or(ErrorCode::Overflow)?;

            referrer_data.daily_referral_rewards = referrer_data
                .daily_referral_rewards
                .checked_add(commission)
                .ok_or(ErrorCode::Overflow)?;
        }

        let mut cursor = std::io::Cursor::new(referrer_account);

        StakingAccount::try_serialize(&referrer_data, &mut cursor)
            .map_err(|_| ErrorCode::SerializationError)?;
    }

    Ok(())
}
