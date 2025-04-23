use crate::errors::ErrorCode;
use crate::state::{GlobalConfig, StakingAccount, UserStakedAccount};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

pub fn handler<'a>(
    ctx: Context<'_, '_, 'a, 'a, Stake<'a>>,
    user_amount: u64,
) -> Result<()> {
    if user_amount == 0 {
        return Err(ErrorCode::InvalidAmount.into());
    }

    let global_config = &mut ctx.accounts.global_config;
    let staking_account = &mut ctx.accounts.staking_account;
    let user_staked = &mut ctx.accounts.user_staked_account;

    let treasury =
        get_associated_token_address(&global_config.treasury, &global_config.mint);

    if treasury != ctx.accounts.treasury_ata.key().clone() {
        return Err(ErrorCode::InvalidTreasury.into());
    }

    msg!(
        "Current APY from GlobalConfig: {}",
        global_config.current_apy_bps
    );

    let deposit_tax = user_amount
        .checked_mul(global_config.deposit_fee_bps as u64)
        .ok_or(ErrorCode::Overflow)?
        .checked_div(10_000)
        .ok_or(ErrorCode::Overflow)?;

    let net_amount = user_amount
        .checked_sub(deposit_tax)
        .ok_or(ErrorCode::Underflow)?;

    // Transfer deposit fee to treasury
    let cpi_ctx_tax = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.treasury_ata.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(cpi_ctx_tax, deposit_tax)?;

    // Burn the net amount
    let cpi_ctx_burn = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            from: ctx.accounts.user_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::burn(cpi_ctx_burn, net_amount)?;

    let remaining_accounts = ctx.remaining_accounts;

    if remaining_accounts.len() > 0 {
        if remaining_accounts.len() > global_config.lvl_percentages.len() {
            return Err(ErrorCode::OnlyTenReferralsLevel.into());
        }

        apply_referral_rewards_for_all_referrer(
            remaining_accounts,
            global_config,
            net_amount,
            staking_account.referrer
        )?;
    } else if staking_account.referrer != Pubkey::default() {
        return Err(ErrorCode::ShouldProvideReferrer.into());
    }

    user_staked.amount_staked = net_amount;

    msg!("Amount staked: {}", net_amount);

    let now = Clock::get()?.unix_timestamp;

    user_staked.last_claimed_timestamp = now;
    user_staked.staked_timestamp = now;
    user_staked.identifier = staking_account.user_staked_counter;
    user_staked.user = staking_account.user;

    staking_account.total_staked = staking_account
        .total_staked
        .checked_add(net_amount)
        .ok_or(ErrorCode::Overflow)?;

    staking_account.user_staked_counter = staking_account
        .user_staked_counter
        .checked_add(1)
        .ok_or(ErrorCode::Overflow)?;

    global_config.total_staked = global_config
        .total_staked
        .checked_add(net_amount)
        .ok_or(ErrorCode::Overflow)?;

    Ok(())
}

pub fn stake_airdrop<'a>(
    ctx: Context<'_, '_, 'a, 'a, Stake<'a>>,
    user_amount: u64,
) -> Result<()> {
    if user_amount == 0 {
        return Err(ErrorCode::InvalidAmount.into());
    }

    let global_config = &mut ctx.accounts.global_config;
    let staking_account = &mut ctx.accounts.staking_account;
    let user_staked = &mut ctx.accounts.user_staked_account;
    let signer_key = ctx.accounts.user.key();

    require!(signer_key == global_config.admin, ErrorCode::OnlyAdmin);
    require!(!staking_account.claim_airdrop, ErrorCode::AirdropAlreadyClaimed);

    msg!(
        "Current APY from GlobalConfig: {}",
        global_config.current_apy_bps
    );

    let net_amount = user_amount;

    let remaining_accounts = ctx.remaining_accounts;

    if remaining_accounts.len() > 0 {
        if remaining_accounts.len() > global_config.lvl_percentages.len() {
            return Err(ErrorCode::OnlyTenReferralsLevel.into());
        }

        apply_referral_rewards_for_all_referrer(
            remaining_accounts,
            global_config,
            net_amount,
            staking_account.referrer
        )?;
    } else if staking_account.referrer != Pubkey::default() {
        return Err(ErrorCode::ShouldProvideReferrer.into());
    }

    user_staked.amount_staked = net_amount;

    msg!("Amount staked: {}", net_amount);

    let now = Clock::get()?.unix_timestamp;

    user_staked.last_claimed_timestamp = now;
    user_staked.staked_timestamp = now;
    user_staked.identifier = staking_account.user_staked_counter;
    user_staked.user = staking_account.user;

    staking_account.total_staked = staking_account
        .total_staked
        .checked_add(net_amount)
        .ok_or(ErrorCode::Overflow)?;

    staking_account.user_staked_counter = staking_account
        .user_staked_counter
        .checked_add(1)
        .ok_or(ErrorCode::Overflow)?;

    global_config.total_staked = global_config
        .total_staked
        .checked_add(net_amount)
        .ok_or(ErrorCode::Overflow)?;

    staking_account.claim_airdrop = true;

    Ok(())
}

pub fn stake_by_admin<'a>(
    ctx: Context<'_, '_, 'a, 'a, Stake<'a>>,
    user_amount: u64,
) -> Result<()> {
    if user_amount == 0 {
        return Err(ErrorCode::InvalidAmount.into());
    }

    let global_config = &mut ctx.accounts.global_config;
    let staking_account = &mut ctx.accounts.staking_account;
    let user_staked = &mut ctx.accounts.user_staked_account;
    let signer_key = ctx.accounts.user.key();

    require!(signer_key == global_config.admin, ErrorCode::OnlyAdmin);

    msg!(
        "Current APY from GlobalConfig: {}",
        global_config.current_apy_bps
    );

    let net_amount = user_amount;

    let remaining_accounts = ctx.remaining_accounts;

    if remaining_accounts.len() > 0 {
        if remaining_accounts.len() > global_config.lvl_percentages.len() {
            return Err(ErrorCode::OnlyTenReferralsLevel.into());
        }

        apply_referral_rewards_for_all_referrer(
            remaining_accounts,
            global_config,
            net_amount,
            staking_account.referrer
        )?;
    } else if staking_account.referrer != Pubkey::default() {
        return Err(ErrorCode::ShouldProvideReferrer.into());
    }

    user_staked.amount_staked = net_amount;

    msg!("Amount staked: {}", net_amount);

    let now = Clock::get()?.unix_timestamp;

    user_staked.last_claimed_timestamp = now;
    user_staked.staked_timestamp = now;
    user_staked.identifier = staking_account.user_staked_counter;
    user_staked.user = staking_account.user;

    staking_account.total_staked = staking_account
        .total_staked
        .checked_add(net_amount)
        .ok_or(ErrorCode::Overflow)?;

    staking_account.user_staked_counter = staking_account
        .user_staked_counter
        .checked_add(1)
        .ok_or(ErrorCode::Overflow)?;

    global_config.total_staked = global_config
        .total_staked
        .checked_add(net_amount)
        .ok_or(ErrorCode::Overflow)?;

    staking_account.claim_airdrop = true;

    Ok(())
}

fn apply_referral_rewards_for_all_referrer(
    remaining_accounts: &[AccountInfo],
    global_config: &Account<GlobalConfig>,
    stake_amount: u64,
    first_referrer: Pubkey
) -> Result<()> {
    let mut next_referrer = first_referrer;

    for (index, account_info) in remaining_accounts.iter().enumerate() {
        if next_referrer != account_info.key() {
            return Err(ErrorCode::ReferrerMismatch.into());
        }

        let commission_percentage = global_config.lvl_percentages.get(index).copied().unwrap_or(0);

        let commission = stake_amount
            .checked_mul(commission_percentage as u64)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(100)
            .ok_or(ErrorCode::Overflow)?;

        if commission > 0 {
            let referrer_account = &mut account_info.try_borrow_mut_data()?[..];

            let mut referrer_data: StakingAccount = StakingAccount::try_deserialize(&mut &referrer_account[..])
                .map_err(|_| ErrorCode::DeserializationError)?;

            next_referrer = referrer_data.referrer;

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

            let mut cursor = std::io::Cursor::new(referrer_account);

            StakingAccount::try_serialize(&referrer_data, &mut cursor)
                .map_err(|_| ErrorCode::SerializationError)?;
        }
    }

    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub staking_account: Account<'info, StakingAccount>,

    #[account(
        init,
        payer = user,
        space = 8 + 8 + 8 + 8 + 8 + 32,
        seeds = [b"user-staked", staking_account.key().as_ref(), staking_account.user_staked_counter.to_le_bytes().as_ref()],
        bump
    )]
    pub user_staked_account: Account<'info, UserStakedAccount>,

    #[account(
        mut,
        seeds = [b"mint"],
        bump = global_config.mint_bumps
    )]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
