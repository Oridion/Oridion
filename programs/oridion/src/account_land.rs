use super::*;

#[derive(Accounts)]
pub struct InitLandBook<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + LandBook::INIT_SPACE,
        seeds = [b"land_book"],
        bump
    )]
    pub land_book: Account<'info, LandBook>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct LandBook {
    #[max_len(128)]
    pub tickets: Vec<[u8; 16]>,
}