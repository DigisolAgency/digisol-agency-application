use anchor_lang::prelude::*;
use anchor_spl::token::{
    spl_token::instruction::AuthorityType, Mint, SetAuthority, Token,
};

use crate::state::GlobalConfig;

#[derive(Accounts)]
pub struct RevokeFreezeAuth<'info> {
    #[account(
        has_one = admin,
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

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
    pub admin: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

  pub fn revoke_freeze_auth(ctx: Context<RevokeFreezeAuth>) -> Result<()> {
    let manager_bumps = ctx.accounts.global_config.transfer_manager_bumps.clone();
    let manager_seeds: &[&[&[u8]]] = &[&[b"transfer_manager", &[manager_bumps]]];

    let context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        SetAuthority {
            account_or_mint: ctx.accounts.mint.to_account_info(),
            current_authority: ctx.accounts.transfer_manager.to_account_info(),
        },
        manager_seeds,
    );
    anchor_spl::token::set_authority(context, AuthorityType::FreezeAccount, None)?;

    Ok(())
}
