use crate::{
    constants::constants,
    errors::ErrorCode,
    state::{GlobalConfig, UserStakedAccount},
};

use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut, has_one = user)]
    pub user_staked_account: Account<'info, UserStakedAccount>,

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
    pub treasury_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler<'a>(ctx: Context<'_, '_, '_, 'a, ClaimRewards<'a>>) -> Result<()> {
    let global_config = &ctx.accounts.global_config;
    let user_staked_account = &mut ctx.accounts.user_staked_account;
    let now = Clock::get()?.unix_timestamp;

    if user_staked_account.staked_timestamp + constants::ONE_YEAR_PERIOD < now {
        return Err(ErrorCode::StakeExpired.into());
    }

    if constants::BLACKLIST_ADDRESSES.contains(&user_staked_account.user.to_string().as_str()) {
        return Err(ErrorCode::Blacklisted.into());
    }

    let treasury =
        get_associated_token_address(&ctx.accounts.global_config.treasury, &global_config.mint);

    if treasury != ctx.accounts.treasury_ata.key().clone() {
        return Err(ErrorCode::InvalidTreasury.into());
    }

    let last_claim_time = user_staked_account.last_claimed_timestamp;

    if now - last_claim_time < constants::CLAIM_PERIOD_SECONDS {
        msg!(
            "Claim too soon: You may claimed at {:?}",
            last_claim_time + constants::CLAIM_PERIOD_SECONDS
        );
        return Err(ErrorCode::ClaimTooSoon.into());
    }

    let reward_lamports_u64 = user_staked_account.calculate_rewards(global_config, now)?;

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

    let withdrawal_tax = reward_lamports_u64
        .checked_mul(global_config.withdrawal_fee_bps as u64)
        .ok_or(ErrorCode::Overflow)?
        .checked_div(10_000)
        .ok_or(ErrorCode::Overflow)?;

    let cpi_ctx_tax = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.treasury_ata.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(cpi_ctx_tax, withdrawal_tax)?;

    // Update last claimed timestamp
    let init_stake_time = user_staked_account.staked_timestamp;
    let stake_hours = init_stake_time % constants::CLAIM_PERIOD_SECONDS;
    let today = now - (now % constants::CLAIM_PERIOD_SECONDS);
    let yesterday = today - constants::CLAIM_PERIOD_SECONDS;

    let new_last_claimed_timestamp = if now >= today + stake_hours {
        today + stake_hours
    } else {
        yesterday + stake_hours
    };

    user_staked_account.last_claimed_timestamp = new_last_claimed_timestamp;

    Ok(())
}
