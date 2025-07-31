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


/// Handles common hop details
pub fn hop_pod(pod: &mut Account<Pod>){
    let clock: Clock = Clock::get().unwrap();
    let now = clock.unix_timestamp;
    let land_time = pod.land_at;

    pod.hops += 1;
    pod.last_process = 1; // Last action -> (1 = hop)
    pod.last_process_at = now;

    //Manual, Instant, Delay
    //Since we will be able to trigger hop manually through lambda function,
    //We only update here pods that are a delay type
    if pod.mode == 1 {
        //Depending on the land timestamp, set the next process and hop process timestamp
        if (now + 180) > land_time {
            //Landing is the next action.
            pod.next_process = 1; //1 = land
            pod.next_process_at = land_time;
        } else {
            //Set the next hop processing timestamp
            pod.next_process = 0; //0 = hop
            pod.next_process_at = now + 180;
        }
    }
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

