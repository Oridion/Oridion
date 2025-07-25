use anchor_lang::prelude::*;

pub const MANAGER_PUBKEY: Pubkey = pubkey!("ordnd8TZFYW4k4MeLrR3qSwXMxezL6W3WryUPYTzLQM");
pub const UNIVERSE_PDA_SEED: &[u8] = b"_x0_ORIDION_0x_";
pub const MAX_USER_META_PODS: usize = 50;

pub const PLANET_PDA_SEED_PRE: &[u8] = b"_PLA_";
pub const PLANET_PDA_SEED_POST: &[u8] = b"_NET_";

//Constants for Star seed
pub const STAR_SEED_PRE: &[u8] = b"_INFINITY_";
pub const STAR_SEED_POST: &[u8] = b"_BEYOND_";
pub const MAX_PLANET_TITLE_LENGTH: usize = 10; // 4 + 6 (MAX 6 CHAR)
pub const MAX_DELAY_ALLOWED: u32 = 86400; //24 hours

pub const LOCK_EXPIRE_SECONDS: i64 = 30; // How many seconds locks expire