use anchor_lang::prelude::*;

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
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Insufficient funds")]    
    InsufficientFunds,
    #[msg("Price Overflow")]
    ArithmeticOverflow,
}
