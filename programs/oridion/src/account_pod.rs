use anchor_lang::prelude::*;
use super::*;


#[account]
#[derive(InitSpace)]
pub struct PodMeta {
    pub authority: Pubkey,
    #[max_len(50)]
    pub ids: Vec<u16>,
    pub created_at: i64,
}
impl PodMeta {
    pub fn prune_ids(&mut self) {
        let len = self.ids.len();
        if len > MAX_USER_META_PODS {
            let excess = len - MAX_USER_META_PODS;
            self.ids.drain(0..excess); // 🧹 remove oldest items
        }
    }
}


// Creates a pod account
#[derive(Accounts)]
#[instruction(id: u16)]
pub struct CreatePod<'info> {
    #[account(
        init_if_needed,
        payer = creator,
        space = 8 + PodMeta::INIT_SPACE,
        seeds = [b"pod_meta", creator.key().as_ref()],
        bump
    )]
    pub pod_meta: Account<'info, PodMeta>,
    #[account(
        init,
        payer = creator,
        space = 8 + Pod::INIT_SPACE,
        seeds = [b"pod", creator.key().as_ref(), &id.to_le_bytes()],
        bump
    )]
    pub pod: Account<'info, Pod>,
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut)]
    pub universe: Account<'info,Universe>,
    #[account(mut)]
    pub planet: Account<'info,Planet>,

    /// CHECK: This account is a lamport collector (PDA). It is not deserialized or mutated.
    ///        We verify its address via seeds and ensure it is program-owned when necessary.
    ///        Used only to receive lamports — no data validation required.
    #[account(mut,seeds = [b"treasury"],bump)]
    pub treasury: AccountInfo<'info>,
    pub destination: SystemAccount<'info>,
    pub system_program: Program<'info,System>
}


#[repr(u8)]
#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub enum ActivityAction {
    Launch = 0,
    Hop = 1,
    Star2 = 2,
    Star3 = 3,
    Scatter = 4
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct ActivityEntry {
    pub action: u8,            // 0 = Launch pod, 1 = Hop, 2 = star_2, 3 = star_3, 4 = scatter, 5 = Land
    pub to: [u8;10],            // Destination planet 10 char string
    pub time: i64,             // Unix timestamp
}

// Pod data
#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Pod {
    // 1-byte fields
    pub account_type: u8,
    pub version: u8, //version 1 default
    pub mode: u8,//1 Delay, 2 Instant, 3 Orbit (Manual)
    pub next_process: u8, // (0 'hop'  1 'land')
    pub last_process: u8, //('0 - launch pod', '1 - hop')
    pub is_in_transit: u8, //flag to see if we are between star hop

    // 2-byte
    pub id: u16, // used for star meta hops
    pub hops: u16, //The number of hops

    // 4-byte
    pub delay: u32, // Set delay in seconds

    // 8-byte fields
    pub next_process_at: i64, //Next process timestamp
    pub land_at: i64, //Set landing timestamp
    pub created_at: i64, //Pod launch timestamp
    pub last_process_at: i64, //Last updated timestamp
    pub lamports: u64, //Lamports deposited

    // 32-byte fields
    pub location: Pubkey, // Current planet location
    pub destination: Pubkey, //Destination wallet address

    pub log: [ActivityEntry; 10], //Stores history during the lifespan of the pod
    pub log_index: u8,
}
impl Pod {
    pub fn init_log(&mut self, planet_key: &str, now: i64) {
        let mut entry = ActivityEntry {
            action: ActivityAction::Launch as u8,
            to: [0u8; 10],
            time: now,
        };

        let bytes = planet_key.as_bytes();
        let len = bytes.len().min(10);
        entry.to[..len].copy_from_slice(&bytes[..len]);

        self.log[0] = entry;
        self.log_index = 1;
    }

    pub fn log_activity(&mut self, action: u8, to: &str) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        let mut entry = ActivityEntry {
            action,
            to: [0u8; 10],
            time: now,
        };

        let bytes = to.as_bytes();
        let len = bytes.len().min(10);
        entry.to[..len].copy_from_slice(&bytes[..len]);

        let index = self.log_index as usize;

        if index < 10 {
            self.log[index] = entry;
            self.log_index += 1;
        } else {
            // Shift all entries left by 1
            for i in 1..10 {
                self.log[i - 1] = self.log[i];
            }
            // Place a new entry at the end
            self.log[9] = entry;
        }
        Ok(())
    }
}

//The signer (MANAGER_PUBKEY) must be the Oridion manager key
#[derive(Accounts)]
pub struct PlanetHop<'info> {
    #[account(mut)]
    pub pod: Account<'info, Pod>,
    #[account(mut)]
    pub to_planet: Account<'info,Planet>,
    #[account(mut)]
    pub from_planet: Account<'info,Planet>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}


#[derive(Accounts)]
pub struct LandAccount<'info> {
    #[account(mut, close = manager)]
    pub pod: Account<'info, Pod>,
    #[account(mut)]
    pub from_planet: Account<'info,Planet>,
    #[account(mut, address = pod.destination)]
    pub destination: SystemAccount<'info>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}

// Scatter Hop
#[account]
#[derive(InitSpace)]
pub struct TransitMeta {
    pub from: Pubkey,
    pub to_pdas: [Pubkey; 5],
    pub amounts: [u64; 5],
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct StarMeta {
    pub to_pdas: [Pubkey; 3],
    pub bump: u8,
}


#[derive(Accounts)]
pub struct ScatterStart<'info> {

    #[account(mut)]
    pub pod: Account<'info, Pod>,

    #[account(mut)]
    pub from_planet: Account<'info,Planet>,
    #[account(mut)]
    pub to_planet_1: Account<'info,Planet>,
    #[account(mut)]
    pub to_planet_2: Account<'info,Planet>,
    #[account(mut)]
    pub to_planet_3: Account<'info,Planet>,
    #[account(mut)]
    pub to_planet_4: Account<'info,Planet>,
    #[account(mut)]
    pub to_planet_5: Account<'info,Planet>,

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
    #[account(mut)]
    pub from_planet_4: Account<'info, Planet>,
    #[account(mut)]
    pub from_planet_5: Account<'info, Planet>,

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



//Star hop from Planet to Split stars
#[derive(Accounts)]
#[instruction(star_one_id: String, star_two_id: String)]
pub struct StarHopTwoStart<'info> {
    #[account(mut)]
    pub pod: Account<'info, Pod>,

    #[account(mut)]
    pub from_planet: Account<'info,Planet>,
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
    pub to_planet: Account<'info,Planet>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_one.manager)]
    pub star_one: Account<'info, Star>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_two.manager)]
    pub star_two: Account<'info, Star>,

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
    pub from_planet: Account<'info,Planet>,
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
    pub to_planet: Account<'info,Planet>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_one.manager)]
    pub star_one: Account<'info, Star>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_two.manager)]
    pub star_two: Account<'info, Star>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_three.manager)]
    pub star_three: Account<'info, Star>,
    
    // #[account(mut,close = manager)] Emergency fix
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

#[account]
#[derive(InitSpace)]
pub struct Star {
    pub amount: u64,
    pub manager: Pubkey
}