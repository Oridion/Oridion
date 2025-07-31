use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StarMeta {
    pub to_pdas: [Pubkey; 3],
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Star {
    pub amount: u64,
    pub manager: Pubkey
}
