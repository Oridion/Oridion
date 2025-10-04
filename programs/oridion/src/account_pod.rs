use super::*;

#[account]
#[derive(InitSpace)]
pub struct PodMeta {
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

    pub fn remove_id(&mut self, id: u16) {
        if let Some(pos) = self.ids.iter().position(|x| *x == id) {
            self.ids.remove(pos);
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
    pub destination: [u8; 32], //Destination wallet address

    // 6-byte alphanumeric emergency passcode hash (e.g., "A7X93B")
    pub passcode_hash: [u8; 32],
    pub authority: [u8; 32], //Authority wallet address
}