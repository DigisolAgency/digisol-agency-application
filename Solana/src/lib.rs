use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
use instructions::*;
declare_id!("H8ZGtC3Ds4fik19JXZZLErhPRw4FaZRtjardu7Z4JCUJ");

#[program]
pub mod staking {

    use super::*;

    pub fn set_referrals_info(
        ctx: Context<SetReferralsInfo>,
        new_total_referral_staked: u64,
        new_daily_referral_rewards: u64,
    ) -> Result<()> {
        instructions::set_referrals_info::set_referrals_info(
            ctx,
            new_total_referral_staked,
            new_daily_referral_rewards,
        )
    }

    pub fn transfer_ownership(ctx: Context<TransferOwnership>, new_admin: Pubkey) -> Result<()> {
        instructions::ownable::transfer_ownership(ctx, new_admin)
    }

    pub fn accept_ownership(ctx: Context<AcceptOwnership>) -> Result<()> {
        instructions::ownable::accept_ownership(ctx)
    }

    pub fn initialize(ctx: Context<Initialize>, referrer: Pubkey) -> Result<()> {
        instructions::initialize_stake::handler(ctx, referrer)
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        instructions::mint::mint_tokens(ctx, amount)
    }

    pub fn initialize_global_config(
        ctx: Context<InitializeGlobalConfig>,
        apy: u32,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        referral_percentages: [u8; 10],
    ) -> Result<()> {
        instructions::initialize_config::handler(
            ctx,
            apy,
            deposit_fee_bps,
            withdrawal_fee_bps,
            referral_percentages,
        )
    }

    pub fn update_apy(ctx: Context<UpdateGlobalConfig>, new_apy: u32) -> Result<()> {
        instructions::update_config::update_apy(ctx, new_apy)
    }

    pub fn update_withdrawal_fee(ctx: Context<UpdateGlobalConfig>, new_fee_bps: u16) -> Result<()> {
        instructions::update_config::update_withdrawal_fee(ctx, new_fee_bps)
    }

    pub fn update_deposit_fee(ctx: Context<UpdateGlobalConfig>, new_fee_bps: u16) -> Result<()> {
        instructions::update_config::update_deposit_fee(ctx, new_fee_bps)
    }

    pub fn update_treasury(ctx: Context<UpdateGlobalConfig>, new_treasury: Pubkey) -> Result<()> {
        instructions::update_config::update_treasury(ctx, new_treasury)
    }

    pub fn update_lvl_percentages(
        ctx: Context<UpdateGlobalConfig>,
        new_percentages: [u8; 10],
    ) -> Result<()> {
        instructions::update_config::update_lvl_percentages(ctx, new_percentages)
    }

    pub fn stake<'a>(ctx: Context<'_, '_, 'a, 'a, Stake<'a>>, amount: u64) -> Result<()> {
        instructions::stake::handler(ctx, amount)
    }

    pub fn stake_airdrop<'a>(
        ctx: Context<'_, '_, 'a, 'a, Stake<'a>>,
        amount: u64,
    ) -> Result<()> {
        instructions::stake::stake_airdrop(ctx, amount)
    }

    pub fn stake_by_admin<'a>(
        ctx: Context<'_, '_, 'a, 'a, Stake<'a>>,
        amount: u64,
    ) -> Result<()> {
        instructions::stake::stake_by_admin(ctx, amount)
    }

    pub fn claim_rewards<'a>(ctx: Context<'_, '_, '_, 'a, ClaimRewards<'a>>) -> Result<()> {
        instructions::claim_rewards::handler(ctx)
    }

    pub fn claim_all_rewards<'a>(ctx: Context<'_, '_, '_, 'a, ClaimAllRewards<'a>>) -> Result<()> {
        instructions::claim_all_rewards::handler(ctx)
    }

    pub fn claim_referral_rewards(ctx: Context<ClaimReferralRewards>) -> Result<()> {
        instructions::claim_referral_rewards::handler(ctx)
    }

    pub fn get_referral_rewards(ctx: Context<GetReferralRewards>) -> Result<u64> {
        instructions::get_referral_rewards::calculate_referral_rewards(ctx)
    }

    pub fn get_rewards_by_user(ctx: Context<GetRewardsByUser>) -> Result<u64> {
        instructions::get_rewards_by_user::calculate_rewards(ctx)
    }

    pub fn get_all_rewards_by_user(ctx: Context<GetAllRewardsByUser>) -> Result<u64> {
        instructions::get_all_rewards_by_user::calculate_rewards(ctx)
    }

    pub fn get_current_rewards_by_user(ctx: Context<GetCurrentRewardsByUser>) -> Result<u128> {
        instructions::get_current_rewards_by_user::calculate_current_rewards(ctx)
    }

    pub fn remove_freeze_authority(ctx: Context<RevokeFreezeAuth>) -> Result<()> {
        instructions::remove_freeze_authority::revoke_freeze_auth(ctx)
    }

    pub fn add_referrer(ctx: Context<AddReferrer>, new_referrer: Pubkey) -> Result<()> {
        instructions::add_referrer::add_referrer(ctx, new_referrer)
    }

    pub fn otc_buy<'a>(
        ctx: Context<'_, '_, 'a, 'a, OtcBuy<'a>>,
        sol_amount: u64,
        toon_amount: u64,
    ) -> Result<()> {
        instructions::otc_buy::handler(ctx, sol_amount, toon_amount)
    }
}
