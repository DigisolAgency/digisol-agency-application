use crate::errors::ErrorCode;
use crate::state::{ApyChange, GlobalConfig};
use anchor_lang::prelude::*;
pub fn update_apy(ctx: Context<UpdateGlobalConfig>, new_apy: u32) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    let now = Clock::get()?.unix_timestamp;

    // Ensure the APY is within the valid range
    require!((1_000..=100_000).contains(&new_apy), ErrorCode::InvalidAPY);

    /*
     * Update APY history:
     * apy_bps - previous APY value
     * timestamp - time while previous APY value is working
     */
    global_config.apy_history[0] = ApyChange {
        apy_bps: global_config.current_apy_bps,
        timestamp: now,
    };

    // Update the current APY in the global configuration
    global_config.current_apy_bps = new_apy;

    msg!("APY updated to {} BPS", new_apy);
    Ok(())
}

pub fn update_withdrawal_fee(ctx: Context<UpdateGlobalConfig>, new_fee_bps: u16) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;

    require!(new_fee_bps <= 2_500, ErrorCode::InvalidWithdrawalFee);

    global_config.withdrawal_fee_bps = new_fee_bps;

    msg!("Withdrawal fee updated to {} BPS", new_fee_bps);
    Ok(())
}
pub fn update_lvl_percentages(
    ctx: Context<UpdateGlobalConfig>,
    new_percentages: [u8; 10],
) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;

    require!(
        new_percentages.iter().all(|&p| p <= 100),
        ErrorCode::InvalidLevelPercentage
    );

    global_config.lvl_percentages = new_percentages;

    msg!("Level percentages updated to {:?}", new_percentages);
    Ok(())
}
pub fn update_deposit_fee(ctx: Context<UpdateGlobalConfig>, new_fee_bps: u16) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;

    // Ensure the new deposit fee is within the acceptable range (0% to 25% or 0-2500 BPS)
    require!(new_fee_bps <= 2_500, ErrorCode::InvalidDepositFee);

    // Update the deposit fee in the global configuration
    global_config.deposit_fee_bps = new_fee_bps;

    msg!("Deposit fee updated to {} BPS", new_fee_bps);
    Ok(())
}

pub fn update_treasury(ctx: Context<UpdateGlobalConfig>, new_treasury: Pubkey) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;

    global_config.treasury = new_treasury;

    msg!("Treasury updated to {}", new_treasury);
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateGlobalConfig<'info> {
    #[account(
        mut,
        has_one = admin,
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    pub admin: Signer<'info>,
}
