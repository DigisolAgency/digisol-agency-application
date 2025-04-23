use crate::{constants::constants, errors::ErrorCode, state::{GlobalConfig, StakingAccount}};
use anchor_lang::prelude::*;

pub fn handler(ctx: Context<Initialize>, referrer: Pubkey) -> Result<()> {
    let user_key = ctx.accounts.user_key.key();
    let signer_key = ctx.accounts.user.key();

    if user_key != signer_key {
     require!(signer_key == ctx.accounts.global_config.admin, ErrorCode::OnlyAdmin);
    }

    let referrer_exist = referrer != Pubkey::default();

    if referrer_exist && !ctx.accounts.referrer_staking_account.is_initialized() {
        return Err(ErrorCode::ReferrerNotInitialized.into());
    }

    let staking_account = &mut ctx.accounts.staking_account;

    if referrer == user_key {
        return Err(ErrorCode::ReferrerIsUser.into());
    }

    staking_account.user = user_key;
    staking_account.total_staked = 0;
    staking_account.referral_history = Vec::new();
    staking_account.user_staked_counter = 0;
    staking_account.last_referral_rewards_claimed = Clock::get()?.unix_timestamp;

    let referrer_staking_account = &mut ctx.accounts.referrer_staking_account;

    if referrer_exist {
        staking_account.referrer = referrer_staking_account.key();
    }

    if referrer_staking_account.referral_history.len() < constants::MAX_REFERRAL && referrer_exist {
        referrer_staking_account.referral_history.push(staking_account.key());
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(referrer: Pubkey)]
pub struct Initialize<'info> {
    #[account(
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        init,
        payer = user,
        space = StakingAccount::LEN,
        seeds = [b"staking", user_key.key().as_ref()],
        bump
    )]
    pub staking_account: Account<'info, StakingAccount>,

    #[account(
        init_if_needed,
        payer = user,
        space = StakingAccount::LEN,
        seeds = [b"staking", referrer.key().as_ref()],
        bump
    )]
    pub referrer_staking_account: Account<'info, StakingAccount>,

    /// CHECK: user account
    pub user_key: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
