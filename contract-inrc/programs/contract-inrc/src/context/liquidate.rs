use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::{
    Config, UserCollateral, SEED_CONFIG_ACCOUNT, SEED_MINT_ACCOUNT, SEED_TREASURY_AUTHORITY, SEED_COLLATERAL_ACCOUNT, USDC_INR_FEED_ID_BYTES,
};

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(
        seeds = [SEED_CONFIG_ACCOUNT],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [SEED_MINT_ACCOUNT],
        bump,
    )]
    pub inrc_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = inrc_mint,
        associated_token::authority = liquidator,
    )]
    pub liquidator_inrc_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = config.usdc_mint,
        associated_token::authority = liquidator,
    )]
    pub liquidator_usdc_account: Account<'info, TokenAccount>,

    /// CHECK: This is a PDA for the mint authority
    #[account(
        seeds = [SEED_TREASURY_AUTHORITY],
        bump = config.treasury_authority_bump,
    )]
    pub treasury_authority: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = config.usdc_mint,
        associated_token::authority = treasury_authority,
    )]
    pub treasury_usdc_account: Account<'info, TokenAccount>,

    /// CHECK: This is the original depositor to
    /// be liquidated
    pub user_to_liquidate: AccountInfo<'info>, 

    #[account(
        mut,
        seeds = [SEED_COLLATERAL_ACCOUNT, user_to_liquidate.key().as_ref()], 
        bump = user_collateral.bump,
    )]
    pub user_collateral: Account<'info, UserCollateral>,

    /// CHECK: This is a price feed
    #[account(
        address = Pubkey::new_from_array(USDC_INR_FEED_ID_BYTES),
    )]
    pub usdc_inr_price_feed: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
}