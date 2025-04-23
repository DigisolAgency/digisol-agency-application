use crate::constants;
use crate::state::GlobalConfig;
use crate::{errors::ErrorCode, state::ApyChange};

use ::{
    anchor_lang::prelude::*,
    anchor_spl::{
        metadata::{
            create_metadata_accounts_v3, mpl_token_metadata::types::DataV2,
            CreateMetadataAccountsV3, Metadata,
        },
        token::{Mint, Token},
    },
};

pub fn handler(
    ctx: Context<InitializeGlobalConfig>,
    apy: u32,
    deposit_fee_bps: u16,
    withdrawal_fee_bps: u16,
    referral_percentages: [u8; 10],
) -> Result<()> {
    // Validate APY range
    require!((1_000..=100_000).contains(&apy), ErrorCode::InvalidAPY);

    // Validate deposit fee (max 25%)
    require!(deposit_fee_bps <= 2_500, ErrorCode::InvalidDepositFee);

    // Validate withdrawal fee (max 25%)
    require!(withdrawal_fee_bps <= 2_500, ErrorCode::InvalidWithdrawalFee);

    // Validate referral percentages
    require!(
        referral_percentages.iter().all(|&p| p <= 100),
        ErrorCode::InvalidLevelPercentage
    );

    let global_config = &mut ctx.accounts.global_config;
    global_config.config_bumps = ctx.bumps.global_config;
    global_config.transfer_manager_bumps = ctx.bumps.transfer_manager;
    global_config.mint_bumps = ctx.bumps.mint;
    global_config.transfer_manager = ctx.accounts.transfer_manager.key();
    global_config.mint = ctx.accounts.mint.key();
    global_config.current_apy_bps = apy;
    global_config.deposit_fee_bps = deposit_fee_bps;
    global_config.withdrawal_fee_bps = withdrawal_fee_bps;
    global_config.lvl_percentages = referral_percentages;
    global_config.treasury = ctx.accounts.treasury.key();
    global_config.admin = ctx.accounts.authority.key();

    let now = Clock::get()?.unix_timestamp;
    global_config.apy_history.push(ApyChange {
        apy_bps: apy,
        timestamp: now,
    });

    let signer_seeds: &[&[&[u8]]] =
        &[&[b"transfer_manager", &[global_config.transfer_manager_bumps]]];

    create_metadata_accounts_v3(
        CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.transfer_manager.to_account_info(),
                update_authority: ctx.accounts.transfer_manager.to_account_info(),
                payer: ctx.accounts.authority.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        DataV2 {
            name: constants::TOKEN_NAME.to_string(),
            symbol: constants::TOKEN_SYMBOL.to_string(),
            uri: constants::TOKEN_URI.to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        false,
        true,
        None,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeGlobalConfig<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [b"config"],
        space = GlobalConfig::LEN,
        bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    /// CHECK: empty PDA, will be set as manager for token accounts
    #[account(
        init,
        seeds = [b"transfer_manager"],
        bump,
        payer = authority,
        space = 0,
    )]
    pub transfer_manager: AccountInfo<'info>,

    /// CHECK:
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            mint.key().as_ref()
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,

    #[account(
        init,
        seeds = [b"mint"],
        bump,
        payer = authority,
        mint::authority = transfer_manager,
        mint::freeze_authority = transfer_manager,
        mint::decimals = 6,
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: treasury account
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
