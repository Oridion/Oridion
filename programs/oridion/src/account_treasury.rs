use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Treasury::INIT_SPACE,
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFromTreasury<'info> {
    #[account(mut)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub recipient: SystemAccount<'info>,
    #[account(mut, address = treasury.authority)]
    pub manager: Signer<'info>, 
    pub system_program: Program<'info, System>,
}

#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Treasury {
    pub bump: u8,
    pub authority: Pubkey
}