use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, MintTo, Token, TokenAccount};

use crate::{constants, errors::ErrorCode, state::GlobalConfig};

pub fn mint_tokens(ctx: Context<MintToken>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidMintAmount);

    require!(
        amount + ctx.accounts.mint.supply <= constants::MAX_SUPPLY,
        ErrorCode::MaxSupplyReached
    );

    let manager_bumps = ctx.accounts.global_config.transfer_manager_bumps.clone();
    let manager_seeds: &[&[&[u8]]] = &[&[b"transfer_manager", &[manager_bumps]]];

    let context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.associated_token_account.to_account_info(),
            authority: ctx.accounts.transfer_manager.to_account_info(),
        },
        manager_seeds,
    );
    anchor_spl::token::mint_to(context, amount)?;

    msg!("Minted {} tokens.", amount);

    Ok(())
}

#[derive(Accounts)]
pub struct MintToken<'info> {
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
    #[account(mut)]
    pub associated_token_account: Account<'info, TokenAccount>,
    /// CHECK: The SPL Token Program.
    pub token_program: Program<'info, Token>,
}
