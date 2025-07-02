use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount, MintTo, mint_to};


pub const LIQUIDATION_BONUS: u64 = 5;
pub const LIQUIDATION_THRESHOLD: u64 = 150; 
pub const MINT_DECIMAL: u8 = 6; 
pub const MIN_HEALTH_FACTOR: u64 = 120; 
pub const MAX_AGE: u64 = 60; 
pub const PRICE_FEED_DECIMAL_ADJUSTMENT: u128 = 100_000_000; 

pub const SEED_CONFIG_ACCOUNT: &[u8] = b"config";
pub const SEED_MINT_ACCOUNT: &[u8] = b"inrc_mint";
pub const SEED_TREASURY_AUTHORITY: &[u8] = b"treasury_authority";
pub const SEED_COLLATERAL_ACCOUNT: &[u8] = b"user_collateral";
pub const USDC_INR_FEED_ID: &str = "0x2d3a776c7c2e4f014168c07e0b57e7a7f45b7e8d641d4c2b92d6e3f5b7e8d641";


declare_id!("FNKmejvZ2Gx3Rjut2MKoqxcz8M8HToMiQnazjDtMcYRY");

#[program]
pub mod contract_new {
    use anchor_spl::token::{self, Transfer};

    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        ctx.accounts.config.authority = ctx.accounts.signer.key();
        ctx.accounts.config.inrc_mint = ctx.accounts.inrc_mint.key();
        ctx.accounts.config.usdc_mint = ctx.accounts.usdc_mint.key();
        ctx.accounts.config.treasury_authority = ctx.accounts.treasury_authority.key();
        ctx.accounts.config.bump = ctx.bumps.config;
        ctx.accounts.config.treasury_authority_bump = ctx.bumps.treasury_authority;
        ctx.accounts.config.mint_pda_bump = ctx.bumps.inrc_mint;
        msg!("Initializing config");
        
        Ok(())
    }

    pub fn deposit_usdc_and_mint_inrc(ctx: Context<DepositUsdcAndMintInrc>, amount_usdc: u64) -> Result<()> {
        let config = &mut ctx.accounts.config;
        let user_collateral = &mut ctx.accounts.user_collateral;
        

        if amount_usdc == 0 {
            return err!(ErrorCode::InvalidAmount);
        }

        if user_collateral.depositor == Pubkey::default() {
            user_collateral.depositor = ctx.accounts.signer.key();
            user_collateral.bump = ctx.bumps.user_collateral;
            user_collateral.usdc_deposit = 0;
            user_collateral.inrc_minted = 0;
            msg!("User collateral account created for {}", user_collateral.depositor);
        }else if  user_collateral.depositor != ctx.accounts.signer.key() {
            return err!(ErrorCode::Unauthorized);
        }

        let usdc_inr_price = 80;

        if ctx.accounts.user_usdc_account.amount < amount_usdc {
            return err!(ErrorCode::InsufficientFunds);
        }
        
        let total_usdc_after_deposit = user_collateral.usdc_deposit + amount_usdc;

        let total_inrc_value_after_deposit = total_usdc_after_deposit
        .checked_mul(usdc_inr_price)
        .ok_or(ProgramError::ArithmeticOverflow)?;


        // min health factor here is 120 .One
        // should have 120% of required usdc for
        // minting the token
        let max_inrc_to_mint = total_inrc_value_after_deposit
            .checked_mul(100)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(config.min_health_factor)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let inrc_to_mint = max_inrc_to_mint.
            checked_sub(user_collateral.inrc_minted)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        //transfer usdc from user to treasury

        let cpi_account = Transfer {
            from: ctx.accounts.user_usdc_account.to_account_info(),
            to: ctx.accounts.usdc_treasury_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();

        token::transfer(
            CpiContext::new(
                cpi_program.clone(),
                cpi_account,             
                ),
                amount_usdc
        )?;

        // mint inrc to user ata

        if inrc_to_mint > 0 {
            let mint_to_account = MintTo {
                mint: ctx.accounts.inrc_mint.to_account_info(),
                to: ctx.accounts.user_inrc_account.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            };
            
            let trasury_authority_seeds = &[SEED_TREASURY_AUTHORITY,&[config.treasury_authority_bump]];

            let signer_seeds = &[&trasury_authority_seeds[..]];

            mint_to(
                CpiContext::new_with_signer(
                    cpi_program,
                    mint_to_account,
                    signer_seeds,
                ),
                inrc_to_mint
            )?;

        }
        user_collateral.usdc_deposit = total_usdc_after_deposit;
        user_collateral.inrc_minted = user_collateral.inrc_minted.checked_add(inrc_to_mint).ok_or(ProgramError::ArithmeticOverflow)?;

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

#[derive(Accounts)]
pub struct DepositUsdcAndMintInrc<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
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

    pub usdc_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BurnInrcAndWithdrawUsdc<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
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
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = treasury_authority,
    )]
    pub usdc_treasury_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = inrc_mint,
        associated_token::authority = signer,
    )]
    pub user_inrc_account: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
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
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Insufficient funds")]    
    InsufficientFunds,
    #[msg("Price Overflow")]
    ArithmeticOverflow,
}
