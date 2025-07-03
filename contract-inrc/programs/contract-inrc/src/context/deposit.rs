

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;

use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    Config, UserCollateral, SEED_CONFIG_ACCOUNT, SEED_MINT_ACCOUNT, SEED_TREASURY_AUTHORITY, SEED_COLLATERAL_ACCOUNT, USDC_INR_FEED_ID_BYTES,
};

#[derive(Accounts)]
pub struct DepositUsdcAndMintInrc<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
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
        associated_token::mint = usdc_mint,
        associated_token::authority = signer,
    )]
    pub user_usdc_account: Account<'info, TokenAccount>,

    /// CHECK: This is a PDA for the mint authority
    #[account(
        seeds = [SEED_TREASURY_AUTHORITY],
        bump = config.treasury_authority_bump,
    )]
    pub treasury_authority: AccountInfo<'info>,

    #[account(
        init_if_needed, 
        payer = signer,
        seeds = [SEED_COLLATERAL_ACCOUNT, signer.key().as_ref()], 
        bump,
        space = 8 + UserCollateral::INIT_SPACE, 
    )]
    pub user_collateral: Account<'info, UserCollateral>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = usdc_mint,
        associated_token::authority = treasury_authority,
    )]
    pub usdc_treasury_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = inrc_mint,
        associated_token::authority = signer,
    )]
    pub user_inrc_account: Account<'info, TokenAccount>,

    /// CHECK: This is a price feed
     #[account(
        address = Pubkey::new_from_array(USDC_INR_FEED_ID_BYTES),
    )]
    pub usdc_inr_price_feed: AccountInfo<'info>,

    pub usdc_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}
