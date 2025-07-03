use anchor_lang::prelude::*;
use hex_literal::hex;

#[constant]
pub const LIQUIDATION_BONUS: u64 = 5;
pub const LIQUIDATION_THRESHOLD: u64 = 150; 
pub const MINT_DECIMAL: u8 = 6; 
pub const MIN_HEALTH_FACTOR: u64 = 120; 
pub const MAX_AGE: u64 = 60; 
pub const TARGET_PRICE_DECIMALS: i32 = 8;
pub const SEED_CONFIG_ACCOUNT: &[u8] = b"config";
pub const SEED_MINT_ACCOUNT: &[u8] = b"inrc_mint";
pub const SEED_TREASURY_AUTHORITY: &[u8] = b"treasury_authority";
pub const SEED_COLLATERAL_ACCOUNT: &[u8] = b"user_collateral";
pub const USDC_INR_FEED_ID_BYTES: [u8; 32] = hex!["2d3a776c7c2e4f014168c07e0b57e7a7f45b7e8d641d4c2b92d6e3f5b7e8d641"];