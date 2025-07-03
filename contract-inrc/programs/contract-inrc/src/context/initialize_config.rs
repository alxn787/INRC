use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::{
    Config, SEED_CONFIG_ACCOUNT, SEED_MINT_ACCOUNT, SEED_TREASURY_AUTHORITY,MINT_DECIMAL,
};

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
        mint::authority = treasury_authority ,
        mint::freeze_authority = treasury_authority,
        mint::token_program = token_program,
    )]
    pub inrc_mint: Account<'info, Mint>,

    pub usdc_mint: Account<'info, Mint>,

    /// CHECK: This is a PDA for the mint authority
    #[account(
        seeds = [SEED_TREASURY_AUTHORITY],
        bump,
    )]
    pub treasury_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
