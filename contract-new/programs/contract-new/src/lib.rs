use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

pub const LIQUIDATION_BONUS: u64 = 5;
pub const LIQUIDATION_THRESHOLD: u64 = 150; 
pub const MINT_DECIMAL: u8 = 6; 
pub const MIN_HEALTH_FACTOR: u64 = 120; 
pub const MAX_AGE: u64 = 60; 
pub const PRICE_FEED_DECIMAL_ADJUSTMENT: u128 = 100_000_000; 

// Seeds
pub const SEED_CONFIG_ACCOUNT: &[u8] = b"config";
pub const SEED_MINT_ACCOUNT: &[u8] = b"inrc_mint";
pub const SEED_TREASURY_AUTHORITY: &[u8] = b"treasury_authority";
pub const SEED_COLLATERAL_ACCOUNT: &[u8] = b"user_collateral";
pub const USDC_INR_FEED_ID: &str = "0x2d3a776c7c2e4f014168c07e0b57e7a7f45b7e8d641d4c2b92d6e3f5b7e8d641";


declare_id!("FNKmejvZ2Gx3Rjut2MKoqxcz8M8HToMiQnazjDtMcYRY");

#[program]
pub mod contract_new {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        msg!("Initializing config");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [SEED_CONFIG_ACCOUNT],
        bump,
        space = 8 + Config::INIT_SPACE,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = signer,
        seeds = [SEED_MINT_ACCOUNT],
        bump,
        mint::decimals = MINT_DECIMAL,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
        mint::token_program = token_program,
    )]
    pub inrc_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [SEED_TREASURY_AUTHORITY],
        bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}


#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub inrc_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub treasury_authority: Pubkey,
    pub liquidation_threshold: u64,
    pub liquidation_bonus: u64,
    pub min_health_factor: u64,
    pub bump: u8,
    pub treasury_authority_bump: u8,
    pub mint_pda_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserCollateral {
    pub depositor: Pubkey,
    pub usdc_deposit: u64, 
    pub inrc_minted: u64,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Above minimum health factor")]
    AboveMinHealthFactor,
    #[msg("Below minimum health factor")]
    BelowMinHealthFactor,
    #[msg("Price feed not found")]
    InvalidPrice,
    #[msg("Amount to burn is greater than amount minted")] 
    LiquidationAmountTooHigh,
    #[msg("Insufficient collateral to cover liquidation amount")] 
    InsufficientCollateralForLiquidation,
    #[msg("Invalid amount provided. Amount must be greater than zero.")]
    InvalidAmount,
}
