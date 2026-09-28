use solana_sha256_hasher::hashv;
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
        b"ORIDION_HOP_JITTER_V1".as_ref(),
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

    // 1 = Delay, 2 = Instant, 3 = Manual

    // === INSTANT MODE (mode == 2) === //
    if pod.mode == 2 {
        // Move straight to land after this hop
        let tok = token_from(pod.id, pod.lamports, pod.created_at);

        if !book.tickets.iter().any(|t| *t == tok) {
            require!(book.tickets.len() < 128, OridionError::LandBookFull);
            book.tickets.push(tok);
        }

        // Write into the existing [u8;32] field, zero-padded
        write_ticket_into_passcode(&mut pod.passcode_hash, &tok);

        pod.next_process = 1;         // land
        pod.next_process_at = now;    // or now + 1..2s if you prefer a tiny delay
        return Ok(());
    }

    // === DELAY MODE (mode == 1) === //
    if pod.mode == 1 {
        let remaining = land_time.saturating_sub(now);

        if remaining <= 240 {
            // Only transition to LAND once
            if pod.next_process != 1 {
                pod.next_process = 1;  // land
                pod.next_process_at = land_time;

                // Compute land token (binds id+dest+amount+created_at)
                let tok = token_from(pod.id, pod.lamports, pod.created_at);

                // Push only if not already present (idempotent)
                if !book.tickets.iter().any(|t| *t == tok) {
                    // Optional: cap to prevent accidental growth
                    require!(book.tickets.len() < 128, OridionError::LandBookFull);
                    book.tickets.push(tok);
                }

                // Write into the existing [u8;32] field, zero-padded
                write_ticket_into_passcode(&mut pod.passcode_hash, &tok);
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
    if let Ok(clock) = Clock::get() {
        (clock.unix_timestamp % 81 + 10) as u8
    } else {
        50
    }
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

/// The balance that must remain in a planet after any withdrawal.
///
/// `base_lamports` is the operational reserve configured when the planet was
/// created. The runtime rent minimum is checked as well so a stale or too-low
/// configured reserve cannot make the account non-rent-exempt.
pub fn required_planet_reserve(planet: &Account<Planet>) -> Result<u64> {
    let rent_minimum = Rent::get()?.minimum_balance(planet.to_account_info().data_len());
    Ok(planet.base_lamports.max(rent_minimum))
}

/// Fail closed unless `amount` can be withdrawn while preserving the planet's
/// operational/rent reserve. Checked arithmetic also rejects impossible sums.
pub fn require_planet_can_spend(planet: &Account<Planet>, amount: u64) -> Result<()> {
    let required = required_planet_reserve(planet)?
        .checked_add(amount)
        .ok_or(OridionError::UnusualMathError)?;
    require!(
        planet.get_lamports() >= required,
        OridionError::PlanetNotEnoughFundsError
    );
    Ok(())
}

/// Pure form used by off-chain callers and unit tests to mirror the invariant.
#[cfg(test)]
pub fn can_spend_preserving_reserve(
    balance: u64,
    base_lamports: u64,
    rent_minimum: u64,
    amount: u64,
) -> bool {
    base_lamports
        .max(rent_minimum)
        .checked_add(amount)
        .is_some_and(|required| balance >= required)
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
    amount: u64,
    created_at: i64,
) -> [u8;16] {
    let idb = id.to_le_bytes();
    let amb = amount.to_le_bytes();
    let cab = created_at.to_le_bytes();

    // include a domain/version string to future-proof the format
    let digest = hashv(&[
        b"ORIDION_LAND_V1".as_ref(),
        &idb,
        &amb,
        &cab,
    ]).to_bytes();

    digest[0..16].try_into().unwrap()
}


/// Copy a 16-byte ticket into a 32-byte field, zero-padding the rest.
fn write_ticket_into_passcode(passcode_hash: &mut [u8; 32], ticket: &[u8; 16]) {
    // zero first (idempotent and future-proof)
    *passcode_hash = [0u8; 32];
    passcode_hash[..16].copy_from_slice(ticket);
}


pub fn combine_halves(a: [u8; 16], b: [u8; 16]) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[..16].copy_from_slice(&a);
    out[16..].copy_from_slice(&b);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_pod() -> Pod {
        Pod {
            account_type: 1,
            version: 1,
            mode: 1,
            next_process: 0,
            last_process: 0,
            is_in_transit: 0,
            id: 513,
            hops: 7,
            delay: 3600,
            next_process_at: 0,
            land_at: 0,
            created_at: 1_700_000_000,
            last_process_at: 0,
            lamports: 1_234_567_890,
            location: Pubkey::default(),
            destination: [0; 32],
            passcode_hash: [0; 32],
            authority: [0; 32],
        }
    }

    #[test]
    fn land_ticket_hash_matches_pre_upgrade_vector() {
        assert_eq!(
            token_from(513, 1_234_567_890, 1_700_000_000),
            [
                110, 199, 28, 85, 135, 164, 216, 140, 213, 81, 193, 53, 11, 154, 241,
                122,
            ]
        );
    }

    #[test]
    fn spending_preserves_the_greater_of_base_and_rent() {
        assert!(can_spend_preserving_reserve(101, 1, 10, 91));
        assert!(!can_spend_preserving_reserve(100, 1, 10, 91));
        assert!(can_spend_preserving_reserve(101, 10, 1, 91));
        assert!(!can_spend_preserving_reserve(100, 10, 1, 91));
    }

    #[test]
    fn spending_rejects_overflow() {
        assert!(!can_spend_preserving_reserve(u64::MAX, 1, 1, u64::MAX));
    }

    #[test]
    fn hop_jitter_matches_pre_upgrade_vector() {
        assert_eq!(jitter_seconds(&fixture_pod(), 250_000_000, 120, 240), 178);
    }

    #[test]
    fn activity_action_discriminants_remain_explicit() {
        let actions = [
            ActivityAction::Launch,
            ActivityAction::Hop,
            ActivityAction::Star2,
            ActivityAction::Star3,
            ActivityAction::Scatter,
        ];

        for (expected, action) in actions.into_iter().enumerate() {
            let mut encoded = Vec::new();
            action.serialize(&mut encoded).unwrap();
            assert_eq!(encoded, [expected as u8]);
        }
    }
}
