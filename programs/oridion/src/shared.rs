use anchor_lang::solana_program::hash::hashv;
use crate::account_land::LandBook;
use super::*;

#[repr(u8)]
pub enum AccountType {
    Universe = 0,
    Pod = 1,
    Planet = 2
}

/// Helper function to get mode from code.
pub fn mode_string(mode: u8) -> &'static str {
    match mode {
        1 => "Delayed",
        2 => "Instant",
        3 => "Manual",
        _ => "Unknown",
    }
}


// Map a hash to a jitter in [min_s, max_s] (inclusive)
fn jitter_seconds(pod: &Pod, slot: u64, min_s: i64, max_s: i64) -> i64 {
    let idb = pod.id.to_le_bytes();
    let hopsb = pod.hops.to_le_bytes();
    let cab = pod.created_at.to_le_bytes();
    let slotb = slot.to_le_bytes();

    let h = hashv(&[
        b"ORIDION_HOP_JITTER_V1",
        &idb,
        &hopsb,
        &cab,
        &slotb,
    ]).to_bytes();

    let r = u64::from_le_bytes(h[0..8].try_into().unwrap());
    let span = (max_s - min_s + 1) as u64; // e.g., 121
    min_s + (r % span) as i64
}

/// Handles common hop details
pub fn hop_pod(pod: &mut Account<Pod>, book: &mut Account<LandBook>) -> Result<()> {
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    let land_time = pod.land_at;

    pod.hops = pod.hops.saturating_add(1);
    pod.last_process = 1; // hop
    pod.last_process_at = now;

    // Delay mode only
    if pod.mode == 1 {
        let remaining = land_time.saturating_sub(now);

        if remaining <= 240 {
            // Only transition to LAND once
            if pod.next_process != 1 {
                pod.next_process = 1;  // land
                pod.next_process_at = land_time;

                // Compute land token (binds id+dest+amount+created_at)
                let tok = token_from(pod.id, &pod.destination, pod.lamports, pod.created_at);

                // Push only if not already present (idempotent)
                if !book.tickets.iter().any(|t| *t == tok) {
                    // Optional: cap to prevent accidental growth
                    require!(book.tickets.len() < 128, OridionError::LandBookFull);
                    book.tickets.push(tok);
                }
            }
        } else {
            // Randomize the next hop between 2–4 minutes
            let jitter = jitter_seconds(pod, clock.slot, 120, 240);
            let mut next = now.saturating_add(jitter);

            // Clamp so we don't schedule past the planned land time
            if next >= land_time {
                // a few seconds before land to ensure we cross the threshold next time
                next = land_time.saturating_sub(5);
            }
            pod.next_process = 0;  // hop
            pod.next_process_at = next;
        }
    }
    Ok(())
}


/// Generate random percent (integer) between 10 - 90
pub fn get_random_percent() -> u8 {
    let clock = Clock::get().unwrap();
    let base = (clock.unix_timestamp % 81 + 10) as u8; // 10 to 90 inclusive
    base
}


/// Checks if a planet is unlocked or the lock has timed out
/// Used for Planet hops and Start hops
/// End hops check to validate is locked below.
pub fn validate_planet_is_usable (
    from: &Planet,
    pod_key: Pubkey
) -> Result<()> {
    // The withdrawing planet must be unlocked to proceed.
    let is_unlocked = from.locked_at == 0;
    let locked_by_pod = from.locked_by == pod_key;

    if is_unlocked || locked_by_pod {
        return Ok(());
    }
    
    //If locked by another planet, then the lock must be expired to use it. 
    if from.locked_at > 0 {
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;
        let lock_age = now - from.locked_at;
        require!(lock_age >= LOCK_EXPIRE_SECONDS, OridionError::PlanetStillLocked);
    }
    Ok(())
}



/// Validates that the withdrawing planet account is locked before any usage.
pub fn validate_planet_locked_by_pod(
    from: &Planet,
    pod_key: Pubkey,
) -> Result<()> {

    // The withdrawing planet must be locked to proceed.
    let is_locked = from.locked_at != 0;
    require!(is_locked, OridionError::PlanetNotLocked);

    // Must be locked by this pod's pda key
    let owns_lock = from.locked_by == pod_key;
    require!(owns_lock, OridionError::NotAuthorizedToHop);

    // Check for lock expiration (in case it sat too long)
    // If expired (more than 30 seconds) must send a new lock request before hopping.
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    let lock_age = now - from.locked_at;
    require!(lock_age <= LOCK_EXPIRE_SECONDS, OridionError::LockExpired);

    Ok(())
}


/// Resets lock and releases it to be used for withdrawal
pub fn release_planet_lock(planet: &mut Planet) -> Result<()> {
    planet.locked_at = 0;
    planet.locked_by = Pubkey::default();
    Ok(())
}

pub fn nonzero_32(x: &[u8; 32]) -> bool {
    x.iter().any(|&b| b != 0)
}


/// Generates land token for the guarantee of unchanged destination.
pub fn token_from(
    id: u16,
    dest: &Pubkey,
    amount: u64,
    created_at: i64,
) -> [u8;16] {
    let idb = id.to_le_bytes();
    let amb = amount.to_le_bytes();
    let cab = created_at.to_le_bytes();
    let destb = dest.as_ref();

    // include a domain/version string to future-proof the format
    let digest = hashv(&[
        b"ORIDION_LAND_V1",
        &idb,
        destb,
        &amb,
        &cab,
    ]).to_bytes();

    digest[0..16].try_into().unwrap()
}