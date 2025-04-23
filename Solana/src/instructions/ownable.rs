use anchor_lang::prelude::*;

use crate::state::GlobalConfig;

#[derive(Accounts)]
pub struct TransferOwnership<'info> {
    #[account(
        mut,
        has_one = admin,
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    pub admin: Signer<'info>,
}

pub fn transfer_ownership(ctx: Context<TransferOwnership>, new_admin: Pubkey) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;

    global_config.pending_admin = new_admin;

    msg!("Ownership transfer started {} {}", ctx.accounts.admin.key(), new_admin);
    Ok(())
}

#[derive(Accounts)]
pub struct AcceptOwnership<'info> {
    #[account(
        mut,
        has_one = pending_admin,
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    pub pending_admin: Signer<'info>,
}

pub fn accept_ownership(ctx: Context<AcceptOwnership>) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    let pending_admin = global_config.pending_admin.key();

    global_config.admin = pending_admin;
    global_config.pending_admin = Pubkey::default();

    msg!("Ownership transferred to {}", pending_admin);
    Ok(())
}
