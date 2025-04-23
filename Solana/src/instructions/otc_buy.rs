use crate::errors::ErrorCode;
use crate::state::{GlobalConfig, StakingAccount, UserStakedAccount};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::Token;

pub fn handler<'a>(
    ctx: Context<'_, '_, 'a, 'a, OtcBuy<'a>>,
    sol_amount: u64,
    toon_amount: u64,
) -> Result<()> {
    if sol_amount == 0 || toon_amount == 0 {
        return Err(ErrorCode::InvalidAmount.into());
    }

    let global_config = &mut ctx.accounts.global_config;
    let staking_account = &mut ctx.accounts.staking_account;
    let user_staked = &mut ctx.accounts.user_staked_account;
    let buyer = ctx.accounts.user.to_account_info();
    let treasury = ctx.accounts.treasury.to_account_info();
    let referrer = ctx.accounts.referrer.to_account_info();
    let sys_program = ctx.accounts.system_program.to_account_info();

    let referrer_key = referrer.key();

    if treasury.key() != global_config.treasury.key() {
        return Err(ErrorCode::InvalidTreasury.into());
    }

    let treasury_amount = sol_amount * 80 / 100;
    let referrer_amount = sol_amount - treasury_amount;

    msg!("Sending proceeds to treasury and referrer");

    send_sol(buyer.clone(), treasury, treasury_amount, sys_program.clone())?;
    msg!("Sent {} SOL to treasury", treasury_amount);

    send_sol(buyer, referrer, referrer_amount, sys_program)?;
    msg!("Sent {} SOL to referrer", referrer_amount);

    let remaining_accounts = ctx.remaining_accounts;

    if remaining_accounts.len() > 0 {
        if remaining_accounts.len() > global_config.lvl_percentages.len() {
            return Err(ErrorCode::OnlyTenReferralsLevel.into());
        }

        apply_referral_rewards_for_all_referrer(
            remaining_accounts,
            global_config,
            toon_amount,
            staking_account.referrer,
            referrer_key,
        )?;
    } else if staking_account.referrer != Pubkey::default() {
        return Err(ErrorCode::ShouldProvideReferrer.into());
    }

    user_staked.amount_staked = toon_amount;

    msg!("Amount staked: {}", toon_amount);

    let now = Clock::get()?.unix_timestamp;

    user_staked.last_claimed_timestamp = now;
    user_staked.staked_timestamp = now;
    user_staked.identifier = staking_account.user_staked_counter;
    user_staked.user = staking_account.user;

    staking_account.total_staked = staking_account
        .total_staked
        .checked_add(toon_amount)
        .ok_or(ErrorCode::Overflow)?;

    staking_account.user_staked_counter = staking_account
        .user_staked_counter
        .checked_add(1)
        .ok_or(ErrorCode::Overflow)?;

    global_config.total_staked = global_config
        .total_staked
        .checked_add(toon_amount)
        .ok_or(ErrorCode::Overflow)?;

    Ok(())
}

fn apply_referral_rewards_for_all_referrer(
    remaining_accounts: &[AccountInfo],
    global_config: &Account<GlobalConfig>,
    stake_amount: u64,
    first_referrer: Pubkey,
    provided_referrer: Pubkey,
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

            if index == 0 && referrer_data.user.key() != provided_referrer {
                return Err(ErrorCode::InvalidReferrer.into());
            }

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

fn send_sol<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    amount: u64,
    sys_program: AccountInfo<'info>,
) -> Result<()> {
    let cpi_context = CpiContext::new(
        sys_program,
        system_program::Transfer { from, to }
    );

    let res = system_program::transfer(cpi_context, amount);

    if res.is_ok() {
        return Ok(());
    } else {
        return Err(ErrorCode::TransferFailed.into());
    }
}

#[derive(Accounts)]
pub struct OtcBuy<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = global_config.config_bumps,
        has_one = admin
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [b"staking", user.key().as_ref()],
        bump
    )]
    pub staking_account: Account<'info, StakingAccount>,

    #[account(
        init,
        payer = user,
        space = 8 + 8 + 8 + 8 + 8 + 32,
        seeds = [b"user-staked", staking_account.key().as_ref(), staking_account.user_staked_counter.to_le_bytes().as_ref()],
        bump
    )]
    pub user_staked_account: Account<'info, UserStakedAccount>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: we do not read or write the data of this account
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: we do not read or write the data of this account
    #[account(mut)]
    pub referrer: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
