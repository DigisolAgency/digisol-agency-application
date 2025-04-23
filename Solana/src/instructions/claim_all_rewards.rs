use crate::{
    constants::constants,
    errors::ErrorCode,
    state::{GlobalConfig, StakingAccount, UserStakedAccount},
};

use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct ClaimAllRewards<'info> {
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
    pub treasury_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,

    /*
    REMAINING_ACCOUNTS: user_staked_accounts (limit 24)
     */
}

pub fn handler<'a>(ctx: Context<'_, '_, '_, 'a, ClaimAllRewards<'a>>) -> Result<()> {
    let global_config = &ctx.accounts.global_config;
    let staking_account = &mut ctx.accounts.staking_account;
    let now = Clock::get()?.unix_timestamp;

    let remaining_accounts = ctx.remaining_accounts;

    let number_of_stakes = staking_account.user_staked_counter;
    let should_provided_accounts = if number_of_stakes <= constants::CLAIM_ALL_LIMIT {
        number_of_stakes
    } else {
        number_of_stakes % constants::CLAIM_ALL_LIMIT
    };

    if 
    remaining_accounts.len() != should_provided_accounts as usize && 
    remaining_accounts.len() != constants::CLAIM_ALL_LIMIT as usize
    {
        return Err(ErrorCode::ShouldProvideStakedAccounts.into());
    }

    if constants::BLACKLIST_ADDRESSES.contains(&staking_account.user.to_string().as_str()) {
        return Err(ErrorCode::Blacklisted.into());
    }

    let treasury =
        get_associated_token_address(&ctx.accounts.global_config.treasury, &global_config.mint);

    if treasury != ctx.accounts.treasury_ata.key().clone() {
        return Err(ErrorCode::InvalidTreasury.into());
    }

    let reward_lamports_u64 = staking_account.calculate_all_rewards(
        remaining_accounts,
        global_config,
        now
    )?;

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

    msg!("Transferred withdrawal tax to treasury.");

    update_timestamps(remaining_accounts, now)?;

    msg!("Updated timestamps for all accounts.");

    Ok(())
}

fn update_timestamps(
    remaining_accounts: &[AccountInfo],
    now: i64,
) -> Result<()> {
    for account_info in remaining_accounts.iter() {
        let user_staked_account = &mut account_info.try_borrow_mut_data()?[..];
        let mut user_staked_account_data = UserStakedAccount::try_deserialize(&mut &user_staked_account[..])
            .map_err(|_| ErrorCode::DeserializationError)?;

        let init_stake_time = user_staked_account_data.staked_timestamp;
        let stake_hours = init_stake_time % constants::CLAIM_PERIOD_SECONDS;
        let today = now - (now % constants::CLAIM_PERIOD_SECONDS);
        let yesterday = today - constants::CLAIM_PERIOD_SECONDS;

        let new_last_claimed_timestamp = if now >= today + stake_hours {
            today + stake_hours
        } else {
            yesterday + stake_hours
        };

        user_staked_account_data.last_claimed_timestamp = new_last_claimed_timestamp;

        let mut cursor = std::io::Cursor::new(user_staked_account);

        UserStakedAccount::try_serialize(&user_staked_account_data, &mut cursor)
            .map_err(|_| ErrorCode::SerializationError)?;
    }

    Ok(())
}
