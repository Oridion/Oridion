use anchor_lang::prelude::*;
use super::*;
use crate::errors::OridionError;

/// BIG BANG UNIVERSE PDA
#[derive(Accounts)]
pub struct BigBang<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Universe::INIT_SPACE,
        seeds = [UNIVERSE_PDA_SEED],
        bump
    )]
    pub universe: Account<'info, Universe>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateUniverse<'info> {
    #[account(mut, constraint = universe.locked == 0 @ OridionError::UniverseLocked)]
    pub universe: Account<'info, Universe>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub creator: Signer<'info>
}

#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Universe {
    pub account_type: u8,
    // Indicates whether the fee can be modified. 0 = unlocked, 1 = locked.
    pub locked: u8, //Locked
    pub bump: u8, // Bump
    pub created: i64, //Universe started
    pub last_updated: i64, //Last updated (used for comet random id)
    pub fee: u64, // pod base fee in lamports
    pub increment: u64, // fee increment
}
