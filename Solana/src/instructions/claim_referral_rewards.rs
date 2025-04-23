use crate::{
    constants::constants, errors::ErrorCode, state::{GlobalConfig, StakingAccount}
};

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, MintTo, Token, TokenAccount};

#[derive(Accounts)]
pub struct ClaimReferralRewards<'info> {
    #[account(
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [b"staking", user.key().as_ref()],
        bump
    )]
    pub staking_account: Account<'info, StakingAccount>,

    #[account(
        mut,
        seeds = [b"mint"],
        bump = global_config.mint_bumps
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: empty PDA, manager for token accounts
    #[account(
        seeds = [b"transfer_manager"],
        bump = global_config.transfer_manager_bumps,
    )]
    pub transfer_manager: AccountInfo<'info>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClaimReferralRewards>) -> Result<()> {
    let global_config = &ctx.accounts.global_config;
    let staking_account = &mut ctx.accounts.staking_account;

    if constants::BLACKLIST_ADDRESSES.contains(&staking_account.user.to_string().as_str()) {
        return Err(ErrorCode::Blacklisted.into());
    }

    let now = Clock::get()?.unix_timestamp;

    if now - staking_account.last_referral_rewards_claimed < constants::CLAIM_PERIOD_SECONDS {
        msg!(
            "Claim too soon: Last claimed at {:?}",
            staking_account.last_referral_rewards_claimed
        );
        return Err(ErrorCode::ClaimTooSoon.into());
    }

    let reward_lamports_u64 = staking_account.calculate_referral_rewards(global_config, now)?;

    if reward_lamports_u64 == 0 {
        msg!("No rewards available to claim.");
        return Ok(());
    }

    msg!("Calculated rewards: {:?}", reward_lamports_u64);

    // Mint rewards to the user
    let manager_bumps = ctx.accounts.global_config.transfer_manager_bumps.clone();
    let manager_seeds: &[&[&[u8]]] = &[&[b"transfer_manager", &[manager_bumps]]];

    let context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.transfer_manager.to_account_info(),
        },
    )
    .with_signer(manager_seeds);

    anchor_spl::token::mint_to(context, reward_lamports_u64)?;

    msg!("Minted rewards to user's account.");

    // Update last claimed timestamp
    staking_account.last_referral_rewards_claimed = now - (now % constants::CLAIM_PERIOD_SECONDS);

    Ok(())
}
