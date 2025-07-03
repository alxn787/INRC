pub mod constant;
pub mod states;
pub mod context;
pub mod error;

pub use states::*;
pub use context::*;
pub use constant::*;


use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::Burn;
use anchor_spl::token::Transfer;
use anchor_spl::token::{MintTo, mint_to};
use pyth_sdk_solana::state::SolanaPriceAccount; 


declare_id!("FNKmejvZ2Gx3Rjut2MKoqxcz8M8HToMiQnazjDtMcYRY");


#[program]
pub mod contract_inrc {
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
        let config = & ctx.accounts.config;
        let user_collateral = &mut ctx.accounts.user_collateral;
        let clock = Clock::get()?;
        

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

        let usdc_inr_price = get_pyth_price(&ctx.accounts.usdc_inr_price_feed.to_account_info(), clock.unix_timestamp, MAX_AGE, TARGET_PRICE_DECIMALS)?;

        if ctx.accounts.user_usdc_account.amount < amount_usdc {
            return err!(ErrorCode::InsufficientFunds);
        }
        
        let total_usdc_after_deposit = user_collateral.usdc_deposit.checked_add(amount_usdc).ok_or(ErrorCode::ArithmeticOverflow)?;

        let total_inrc_value_after_deposit = (total_usdc_after_deposit as u128)
        .checked_mul(usdc_inr_price )
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // min health factor here is 120 .One
    // should have 120% of required usdc for
    // minting the token
        let max_inrc_value_in_target_decimals = total_inrc_value_after_deposit
            .checked_mul(100)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(config.min_health_factor as u128)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        //making sure the decimals are same
        let max_inrc_to_mint = max_inrc_value_in_target_decimals
            .checked_div(10u128.pow((TARGET_PRICE_DECIMALS - MINT_DECIMAL as i32) as u32))
            .ok_or(ErrorCode::ArithmeticOverflow)?
            as u64;

        let inrc_to_mint = max_inrc_to_mint.
            checked_sub(user_collateral.inrc_minted )
            .ok_or(ErrorCode::ArithmeticOverflow)?;

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
        user_collateral.inrc_minted = user_collateral.inrc_minted.checked_add(inrc_to_mint).ok_or(ErrorCode::ArithmeticOverflow)?;

        Ok(())
    }

     pub fn burn_inrc_and_withdraw_usdc(ctx: Context<BurnInrcAndWithdrawUsdc>, amount_inrc: u64) -> Result<()> {
        let config = & ctx.accounts.config;
        let user_collateral = &mut ctx.accounts.user_collateral;
        let clock = Clock::get()?;
        
        if amount_inrc == 0 {
            return err!(ErrorCode::InvalidAmount);
        }
        
        if amount_inrc > user_collateral.inrc_minted {
            return err!(ErrorCode::LiquidationAmountTooHigh);
        }

        let usdc_inr_price = get_pyth_price(&ctx.accounts.usdc_inr_price_feed.to_account_info(), clock.unix_timestamp, MAX_AGE, TARGET_PRICE_DECIMALS)?;

        let remaining_inrc = user_collateral.inrc_minted.checked_sub(amount_inrc).ok_or(ErrorCode::ArithmeticOverflow)?;

        let current_usdc_value_in_inr = (user_collateral.usdc_deposit as u128)
            .checked_mul(usdc_inr_price)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        let health_factor_after_withdrawal = if remaining_inrc > 0 {
            current_usdc_value_in_inr
                .checked_mul(100)
                .ok_or(ErrorCode::ArithmeticOverflow)?
                .checked_div(remaining_inrc as u128)
                .ok_or(ErrorCode::ArithmeticOverflow)?
                .checked_mul(10u128.pow((TARGET_PRICE_DECIMALS - MINT_DECIMAL as i32) as u32))
                .ok_or(ErrorCode::ArithmeticOverflow)?
        } else {
            u128::MAX 
        };

        //verifying if its above the health factor
        //in which we minted the inrc.. should be 120%
        if health_factor_after_withdrawal < config.min_health_factor as u128 {
            return err!(ErrorCode::BelowMinHealthFactor);
        };

        let usdc_to_withdraw = (amount_inrc as u128)
            .checked_mul(10u128.pow((TARGET_PRICE_DECIMALS - MINT_DECIMAL as i32) as u32))
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(usdc_inr_price)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            as u64;


        let burn_accounts = Burn {
            mint: ctx.accounts.inrc_mint.to_account_info(),
            from: ctx.accounts.user_inrc_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        token::burn(
            CpiContext::new(
                cpi_program.clone(),
                burn_accounts,
            ),
            amount_inrc
        )?;

        let transfer_cpi_account = Transfer {
            from: ctx.accounts.usdc_treasury_account.to_account_info(),
            to: ctx.accounts.user_usdc_account.to_account_info(),
            authority: ctx.accounts.treasury_authority.to_account_info(),
        };

        let treasury_seeds = &[SEED_TREASURY_AUTHORITY,&[config.treasury_authority_bump]];

        let signer_seeds = &[&treasury_seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                cpi_program,
                transfer_cpi_account,
                signer_seeds,
            ),
            usdc_to_withdraw
        )?;
        user_collateral.usdc_deposit = user_collateral.usdc_deposit.checked_sub(usdc_to_withdraw).ok_or(ErrorCode::ArithmeticOverflow)?;
        user_collateral.inrc_minted = user_collateral.inrc_minted.checked_sub(amount_inrc).ok_or(ErrorCode::ArithmeticOverflow)?;
            
       Ok(())
    }

    pub fn liquidate(ctx: Context<Liquidate>, amount_inrc_to_burn: u64) -> Result<()> {
        let config = & ctx.accounts.config;
        let user_collateral = &mut ctx.accounts.user_collateral;
        let liquidator = & ctx.accounts.liquidator;
        let clock = Clock::get()?;

        let usdc_inr_price = get_pyth_price(&ctx.accounts.usdc_inr_price_feed.to_account_info(), clock.unix_timestamp, MAX_AGE, TARGET_PRICE_DECIMALS)?;

        let usdc_value_in_inr = (user_collateral.usdc_deposit as u128)
            .checked_mul(usdc_inr_price)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        let health_factor = if user_collateral.inrc_minted > 0 {
            usdc_value_in_inr
                .checked_mul(100)
                .ok_or(ErrorCode::ArithmeticOverflow)?
                .checked_div((user_collateral.inrc_minted as u128)
                    .checked_mul(10u128.pow((TARGET_PRICE_DECIMALS - MINT_DECIMAL as i32) as u32))
                    .ok_or(ErrorCode::ArithmeticOverflow)?
                )
                .ok_or(ErrorCode::ArithmeticOverflow)?
        } else {
            u128::MAX 
        };

        if health_factor >= config.liquidation_threshold as u128 {
            return err!(ErrorCode::AboveMinHealthFactor);
        }

        if amount_inrc_to_burn > user_collateral.inrc_minted {
            return err!(ErrorCode::LiquidationAmountTooHigh);
        }

        let usdc_to_liquidator = (amount_inrc_to_burn as u128)
            .checked_mul(100 + config.liquidation_bonus as u128) //bonus is applied here
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_mul(10u128.pow((TARGET_PRICE_DECIMALS - MINT_DECIMAL as i32) as u32)) 
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(100) 
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(usdc_inr_price) 
            .ok_or(ErrorCode::ArithmeticOverflow)?
        as u64;

        if usdc_to_liquidator > user_collateral.usdc_deposit {
            return err!(ErrorCode::InsufficientCollateralForLiquidation);
        }

        let burn_accounts = Burn {
            from: ctx.accounts.liquidator_inrc_account.to_account_info(),
            mint: ctx.accounts.inrc_mint.to_account_info(),
            authority: liquidator.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        token::burn(
            CpiContext::new(
                cpi_program.clone(),
                burn_accounts,
            ),
            amount_inrc_to_burn
        )?;

        let transfer_cpi_account = Transfer {
            from: ctx.accounts.treasury_usdc_account.to_account_info(), 
            to: ctx.accounts.liquidator_usdc_account.to_account_info(),
            authority: ctx.accounts.treasury_authority.to_account_info(),
        };

        let treasury_seeds = &[SEED_TREASURY_AUTHORITY,&[config.treasury_authority_bump]];
        let signer_seeds = &[&treasury_seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                cpi_program,
                transfer_cpi_account,
                signer_seeds,
            ),
            usdc_to_liquidator
        )?;

        user_collateral.usdc_deposit = user_collateral.usdc_deposit
        .checked_sub(usdc_to_liquidator)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

        user_collateral.inrc_minted = user_collateral.inrc_minted
        .checked_sub(amount_inrc_to_burn)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
        Ok(())
    }

}

   
fn get_pyth_price(
    price_account_info: &AccountInfo,
    current_timestamp: i64,
    max_age: u64,
    target_decimals: i32,
    ) -> Result<u128> {
    let price_feed = SolanaPriceAccount::account_info_to_feed(price_account_info)
        .map_err(|_| ErrorCode::InvalidPrice)?;

    let current_price = price_feed
        .get_price_no_older_than(current_timestamp, max_age)
        .ok_or(ErrorCode::InvalidPrice)?;
    let price_val = current_price.price;
    let price_expo = current_price.expo;

    let scaled_price: u128;
    if price_expo < target_decimals {
        let diff = (target_decimals - price_expo) as u32;
        scaled_price = (price_val as u128)
            .checked_mul(10u128.pow(diff))
            .ok_or(ErrorCode::ArithmeticOverflow)?;
    } else if price_expo > target_decimals {
        let diff = (price_expo - target_decimals) as u32;
        scaled_price = (price_val as u128)
            .checked_div(10u128.pow(diff))
            .ok_or(ErrorCode::ArithmeticOverflow)?;
    } else {
        scaled_price = price_val as u128;
    }

    Ok(scaled_price)
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
