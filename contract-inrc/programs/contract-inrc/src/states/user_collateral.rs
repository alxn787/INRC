use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserCollateral {
    pub depositor: Pubkey,
    pub usdc_deposit: u64, 
    pub inrc_minted: u64,
    pub bump: u8,
}