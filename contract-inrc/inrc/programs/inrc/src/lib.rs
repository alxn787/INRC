use anchor_lang::prelude::*;

// Constants
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

declare_id!("HiduwdBgbDqaUAYcM65u6MFo7B7EbiHDNW47rfmymf7J");

#[program]
pub mod inrc {
    use super::*;
    
    pub fn initialize(ctx: Context<InitializeConfig>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        
        // Initialize the config account with values
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.signer.key();
        config.inrc_mint = Pubkey::default(); // Will be set when mint is created
        config.usdc_mint = Pubkey::default(); // Will be set when USDC mint is provided
        config.treasury_authority = Pubkey::default(); // Will be set when treasury is created
        config.liquidation_threshold = LIQUIDATION_THRESHOLD;
        config.liquidation_bonus = LIQUIDATION_BONUS;
        config.min_health_factor = MIN_HEALTH_FACTOR;
        config.bump = ctx.bumps.config;
        config.treasury_authority_bump = 0; // Will be set when treasury PDA is created
        config.mint_pda_bump = 0; // Will be set when mint PDA is created
        
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

    pub system_program: Program<'info, System>,
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
    pub usdc_deposit: u64, // Fixed: more descriptive name
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
    #[msg("Amount to burn is greater than amount minted")] // Fixed typo: "Amoun" -> "Amount"
    LiquidationAmountTooHigh,
    #[msg("Insufficient collateral to cover liquidation amount")] // Fixed: lowercase "liquidation"
    InsufficientCollateralForLiquidation,
    #[msg("Invalid amount provided. Amount must be greater than zero.")]
    InvalidAmount,
}