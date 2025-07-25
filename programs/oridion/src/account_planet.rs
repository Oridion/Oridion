use anchor_lang::prelude::*;
use super::*;


/// PLANETS PDA
#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreatePlanet<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Planet::INIT_SPACE,
        seeds = [
            PLANET_PDA_SEED_PRE,
            name.as_ref(),
            PLANET_PDA_SEED_POST
        ],
        bump
    )]
    pub planet: Account<'info, Planet>,
    #[account(mut)]
    pub universe: Account<'info, Universe>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}


//Once stable, this function should be deleted.
#[derive(Accounts)]
pub struct DeletePlanet<'info> {
    #[account(mut,close = creator,constraint = planet.to_account_info().lamports() <= planet.base_lamports)]
    pub planet: Account<'info, Planet>,
    #[account(mut)]
    pub universe: Account<'info, Universe>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub creator: Signer<'info>,
}


#[derive(Accounts)]
pub struct BalancePlanets<'info> {
    #[account(mut)]
    pub from_planet: Account<'info, Planet>,
    #[account(mut)]
    pub to_planet: Account<'info, Planet>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub creator: Signer<'info>,
}

#[derive(Accounts)]
pub struct LockPlanet<'info> {
    #[account(mut)]
    pub planet: Account<'info, Planet>,
    pub pod: Account<'info, Pod>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub creator: Signer<'info>,
}



#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Planet {
    pub account_type: u8,        // 2 = Planet
    pub bump: u8,                // Bump
    pub created: i64,            // Timestamp when the planet was created
    pub visits: u64,             // Number of visits (for analytics or rules)
    pub base_lamports: u64,      // Minimum balance for self-destruct (or cleanup logic)
    pub locked_at: i64,          // Timestamp of lock, 0 = not locked
    pub locked_by: Pubkey,       // Pod that owns the lock
    #[max_len(10)]
    pub name: String,            // Max 10-char planet ID
}