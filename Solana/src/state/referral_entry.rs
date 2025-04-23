use anchor_lang::prelude::*;

#[derive(AnchorSerialize, Debug, AnchorDeserialize, Clone)]
pub struct ReferralEntry {
    pub referrer: Pubkey,
    pub level: u8,
}
