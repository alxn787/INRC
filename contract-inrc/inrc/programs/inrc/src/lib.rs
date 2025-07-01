use anchor_lang::prelude::*;

declare_id!("HiduwdBgbDqaUAYcM65u6MFo7B7EbiHDNW47rfmymf7J");

#[program]
pub mod inrc {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {

}

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub inrc_mint: Pubkey,
}
