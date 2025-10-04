use anchor_lang::prelude::*;
use crate::account_land::LandBook;
use crate::account_star::{Star, StarMeta};
use super::*;

//The signer (MANAGER_PUBKEY) must be the Oridion manager key
#[derive(Accounts)]
pub struct PlanetHop<'info> {
    #[account(mut)]
    pub pod: Account<'info, Pod>,
    #[account(mut)]
    pub to_planet: Account<'info, Planet>,
    #[account(mut)]
    pub from_planet: Account<'info, Planet>,
    #[account(
        mut,
        seeds = [b"land_book"],
        bump
    )]
    pub book: Account<'info, LandBook>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}

//Star hop from Planet to Split stars
#[derive(Accounts)]
#[instruction(star_one_id: String, star_two_id: String)]
pub struct StarHopTwoStart<'info> {
    #[account(mut)]
    pub pod: Account<'info, Pod>,

    #[account(mut)]
    pub from_planet: Account<'info, Planet>,
    #[account(init, payer = manager, space = 8 + Star::INIT_SPACE,
        seeds = [
            STAR_SEED_PRE,
            star_one_id.as_ref(),
            STAR_SEED_POST
        ],
        bump
    )]
    pub star_one: Account<'info, Star>,
    #[account(init, payer = manager, space = 8 + Star::INIT_SPACE,
        seeds = [
            STAR_SEED_PRE,
            star_two_id.as_ref(),
            STAR_SEED_POST
        ],
        bump
    )]
    pub star_two: Account<'info, Star>,

    #[account(
        init,
        seeds = [b"star_two", pod.key().as_ref()],
        bump,
        payer = manager,
        space = 8 + StarMeta::INIT_SPACE
    )]
    pub star_meta: Account<'info, StarMeta>,

    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

//Return from stars to the destination planet
#[derive(Accounts)]
pub struct StarHopTwoEnd<'info> {
    #[account(mut)]
    pub pod: Account<'info, Pod>,
    #[account(mut)]
    pub to_planet: Account<'info, Planet>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_one.manager)]
    pub star_one: Account<'info, Star>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_two.manager)]
    pub star_two: Account<'info, Star>,

    #[account(
        mut,
        seeds = [b"land_book"],
        bump
    )]
    pub book: Account<'info, LandBook>,

    #[account(
        mut,
        seeds = [b"star_two", pod.key().as_ref()],
        bump = star_meta.bump,
        close = manager
    )]
    pub star_meta: Account<'info, StarMeta>,

    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}


#[derive(Accounts)]
#[instruction(star_one_id: String, star_two_id: String, star_three_id: String )]
pub struct StarHopThreeStart<'info> {
    #[account(mut)]
    pub pod: Account<'info, Pod>,

    #[account(mut)]
    pub from_planet: Account<'info, Planet>,
    #[account(init, payer = manager, space = 8 + Star::INIT_SPACE,
        seeds = [
            STAR_SEED_PRE,
            star_one_id.as_ref(),
            STAR_SEED_POST
        ],
        bump
    )]
    pub star_one: Account<'info, Star>,

    #[account(init, payer = manager, space = 8 + Star::INIT_SPACE,
        seeds = [
            STAR_SEED_PRE,
            star_two_id.as_ref(),
            STAR_SEED_POST
        ],
        bump
    )]
    pub star_two: Account<'info, Star>,

    #[account(init, payer = manager, space = 8 + Star::INIT_SPACE,
        seeds = [
            STAR_SEED_PRE,
            star_three_id.as_ref(),
            STAR_SEED_POST
        ],
        bump
    )]
    pub star_three: Account<'info, Star>,

    /// CHECK: This account is manually created with `invoke_signed` using a verified PDA.
    /// The PDA is derived from [b"star_three", pod.key()], and data is written manually using `.serialize()`.
    /// We ensure the account is the correct target by checking the key before use.
    #[account(mut)]
    pub star_meta: UncheckedAccount<'info>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StarHopThreeEnd<'info> {
    #[account(mut)]
    pub pod: Account<'info, Pod>,
    #[account(mut)]
    pub to_planet: Account<'info, Planet>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_one.manager)]
    pub star_one: Account<'info, Star>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_two.manager)]
    pub star_two: Account<'info, Star>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_three.manager)]
    pub star_three: Account<'info, Star>,

    #[account(
        mut,
        seeds = [b"land_book"],
        bump
    )]
    pub book: Account<'info, LandBook>,

    #[account(
        mut,
        seeds = [b"star_three", pod.key().as_ref()],
        bump = star_meta.bump,
        close = manager
    )]
    pub star_meta: Account<'info, StarMeta>,

    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}




// Scatter Hop
#[account]
#[derive(InitSpace)]
pub struct TransitMeta {
    pub from: Pubkey,
    pub to_pdas: [Pubkey; 3],
    pub amounts: [u64; 3],
    pub created_at: i64,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct ScatterStart<'info> {

    #[account(mut)]
    pub pod: Account<'info, Pod>,

    #[account(mut)]
    pub from_planet: Account<'info, Planet>,
    #[account(mut)]
    pub to_planet_1: Account<'info, Planet>,
    #[account(mut)]
    pub to_planet_2: Account<'info, Planet>,
    #[account(mut)]
    pub to_planet_3: Account<'info, Planet>,
    #[account(
        init,
        seeds = [b"scatter",pod.key().as_ref()],
        bump,
        payer = manager,
        space = 8 + TransitMeta::INIT_SPACE
    )]
    pub scatter_meta: Account<'info, TransitMeta>,

    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>,
    pub system_program: Program<'info, System>,
}

//Return from stars to the destination planet
#[derive(Accounts)]
pub struct ScatterEnd<'info> {
    #[account(mut)]
    pub pod: Account<'info, Pod>,

    #[account(mut)]
    pub to_planet: Account<'info, Planet>,

    #[account(mut)]
    pub from_planet_1: Account<'info, Planet>,
    #[account(mut)]
    pub from_planet_2: Account<'info, Planet>,
    #[account(mut)]
    pub from_planet_3: Account<'info, Planet>,

    #[account(
        mut,
        seeds = [b"land_book"],
        bump
    )]
    pub book: Account<'info, LandBook>,

    #[account(
        mut,
        seeds = [b"scatter", pod.key().as_ref()],
        bump = scatter_meta.bump,
        close = manager
    )]
    pub scatter_meta: Account<'info, TransitMeta>,

    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>,
}


/// Destination is checked through a token and cannot be changed.
#[derive(Accounts)]
pub struct LandAccount<'info> {
    #[account(
        mut,
        seeds = [b"land_book"],
        bump
    )]
    pub book: Account<'info, LandBook>,
    #[account(mut)]
    pub from_planet: Account<'info, Planet>,
    #[account(mut)]
    pub destination: SystemAccount<'info>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}


#[derive(Accounts)]
pub struct ClosePod<'info> {
    #[account(mut)]
    pub pod_meta: Account<'info, PodMeta>,
    #[account(mut,
        close = manager,
        constraint = pod.next_process == 1 @ OridionError::PodCloseError,
        constraint = pod.is_in_transit == 0 @ OridionError::PodCloseError,
    )]
    pub pod: Account<'info, Pod>,
    #[account(
        mut,
        seeds = [b"land_book"],
        bump
    )]
    pub book: Account<'info, LandBook>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>,
}


// Emergency land by creator
#[derive(Accounts)]
#[instruction(id: u16)]
pub struct EmergencyLandByCreator<'info> {
    #[account(
        seeds = [b"pod_meta", creator.key().as_ref()],
        bump
    )]
    pub pod_meta: Account<'info, PodMeta>,
    #[account(
        mut,
        seeds = [b"pod", creator.key().as_ref(), &id.to_le_bytes()],
        bump,
        close = creator
    )]
    pub pod: Account<'info, Pod>,

    #[account(mut)]
    pub from_planet: Account<'info, Planet>,

    /// The creator is the signer who originally derived the pod PDA.
    pub creator: Signer<'info>,

    #[account(mut)]
    pub destination: SystemAccount<'info>,
}


#[derive(Accounts)]
#[instruction(amount_lamports: u64)]
pub struct BalancePlanets<'info> {
    #[account(mut)]
    pub to_planet: Account<'info,Planet>,
    #[account(mut)]
    pub from_planet: Account<'info,Planet>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}




