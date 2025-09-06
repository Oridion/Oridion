#![allow(unexpected_cfgs)]

mod variables;
mod account_universe;
pub mod account_pod;
mod account_planet;
mod account_treasury;
mod account_star;

mod shared;
mod errors;
mod all_accounts;
mod account_land;

use account_pod::*;
use account_planet::*;
use account_universe::*;
use account_treasury::*;
use account_land::*;
use all_accounts::*;
use errors::*;
use shared::*;
use variables::*;

use anchor_lang::prelude::*;
use solana_security_txt::security_txt;

use anchor_lang::solana_program as sp;

declare_id!("ord1qJZ3DB52s9NoG8nuoacW85aCyNvECa5kAqcBVBu");

security_txt! {
    name: "Oridion",
    website: "https://oridion.xyz",
    project_url: "https://oridion.xyz",
    source_code: "https://github.com/Oridion/oridion_anchor",
    preferred_languages: "en",
    contacts: "twitter:@OridionGalaxy,email:oridion.xyz@gmail.com",
    policy: "https://oridion.xyz/privacy-policy"
}

#[program]
pub mod oridion {
    use super::*;
    use anchor_lang::solana_program::system_instruction::transfer;
    use anchor_lang::solana_program::system_instruction::create_account;
    use anchor_lang::solana_program::instruction::Instruction;
    use super::sp::program::{invoke, invoke_signed};
    use crate::account_land::{InitLandBook, LandBook};
    use crate::account_star::{Star, StarMeta};

    /// UNIVERSE
    pub fn bang(ctx: Context<BigBang>) -> Result<()> {
        let clock: Clock = Clock::get()?;
        let universe: &mut Account<Universe> = &mut ctx.accounts.universe;
        universe.account_type = AccountType::Universe as u8;
        universe.locked = 0;
        universe.bump = ctx.bumps.universe; // store bump seed in `Counter` account
        universe.created = clock.unix_timestamp;
        universe.last_updated = clock.unix_timestamp; //must set this here as well for random comet id
        universe.fee = 3000000; //base fee Lamports (0.03 SOL)
        universe.increment = 100000; // Increment per additional (start)
        Ok(())
    }

    /// UPDATE FEE
    pub fn configure(ctx: Context<UpdateUniverse>, fee: u32, increment: u32) -> Result<()> {
        let clock: Clock = Clock::get()?;
        let universe: &mut Account<Universe> = &mut ctx.accounts.universe;
        universe.last_updated = clock.unix_timestamp; //must set this here as well for random comet id
        universe.fee = fee as u64; //Lamports
        universe.increment = increment as u64; //Lamports
        Ok(())
    }

    /// LOCK UNIVERSE
    pub fn seal(ctx: Context<UpdateUniverse>) -> Result<()> {
        let universe = &mut ctx.accounts.universe;
        universe.locked = 1;
        Ok(())
    }


    /// CREATE PLANET
    pub fn register_node(ctx: Context<CreatePlanet>, name: String) -> Result<()> {
        let clock: Clock = Clock::get()?;
        let planet: &mut Account<Planet> = &mut ctx.accounts.planet;
        //Make sure the planet name is not too long
        require!(name.len() <= MAX_PLANET_TITLE_LENGTH, OridionError::PlanetNameTooLong);
        planet.account_type = AccountType::Planet as u8;
        planet.created = clock.unix_timestamp;
        planet.visits = 0;
        planet.bump = ctx.bumps.planet;
        planet.base_lamports = planet.to_account_info().lamports();
        planet.name = name.clone();
        planet.locked_by = Pubkey::default();
        planet.locked_at = 0;
        Ok(())
    }

    /// DELETE PLANET
    pub fn retire_node(ctx: Context<DeletePlanet>) -> Result<()> {
        let planet: &mut Account<Planet> = &mut ctx.accounts.planet;
        require!(
            planet.to_account_info().lamports() <= planet.base_lamports,
            OridionError::PlanetDeleteHasFundsError
        );
        Ok(())
    }

    /// INIT TREASURY
    pub fn init_vault(ctx: Context<InitializeTreasury>) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;
        treasury.bump = ctx.bumps.treasury;
        treasury.authority = ctx.accounts.payer.key();
        Ok(())
    }

    /// INIT LANDING BOOK
    pub fn init_registry(_ctx: Context<InitLandBook>) -> Result<()> {
        Ok(())
    }


    /// WITHDRAW FROM TREASURY
    pub fn payout(ctx: Context<WithdrawFromTreasury>, amount: u64, ) -> Result<()> {
        // Safety: Check if Treasury has enough
        let treasury_balance = **ctx.accounts.treasury.to_account_info().lamports.borrow();
        require!(
            treasury_balance >= amount,
            OridionError::InsufficientTreasuryBalance
        );

        // Move lamports manually
        **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.recipient.to_account_info().try_borrow_mut_lamports()? += amount;
        msg!(
            "Withdrawn {} lamports from treasury to {}",
            amount,
            ctx.accounts.recipient.key()
        );
        Ok(())
    }


    /// # Optional Transparency Memo
    ///
    /// The `show_memo` flag allows the user to optionally attach a public `spl-memo` instruction
    /// to their pod transaction for transparency and wallet compatibility.
    ///
    /// ## Purpose
    /// - Helps wallets like Phantom recognize and explain the transaction.
    /// - Assists users in identifying their activity on explorers.
    /// - Does **not** affect privacy unless explicitly enabled.
    ///
    /// ## Defaults to `false`
    /// - Users must explicitly opt in.
    /// - When `false`, no memo is added. The pod remains private.
    /// - When `true`, a memo like `Oridion: launched pod from 7GdQ...2kHt` is attached.
    ///
    /// ## Security Note
    /// - This setting is respected on-chain; users cannot be forced to reveal origin.
    /// - This pattern maintains compatibility with trust-scoring logic used by wallets.
    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct PodArgs {
        pub deposit_lamports: u64,
        pub mode: u8,
        pub delay: u32,
        pub show_memo: u8,
        pub passcode: [u8; 32],
        pub d0: [u8; 16],
        pub d1: [u8; 16],
    }


    ///-------------------------------------------------------------------///
    /// LAUNCH POD
    /// Creates a user's pod account and handles initial launch to a planet.
    /// - Destination address must be set on creation.
    /// Otherwise, users will not trust Oridion if it can dictate the landing
    /// destination address at any time. This is not safe for the user. User should know
    /// that Oridion cannot alter their deposit or destination.
    ///-------------------------------------------------------------------///
    pub fn launch(ctx: Context<CreatePod>, id: u16, args: PodArgs) -> Result<()> {

        // -------------------------------------------------//
        // ARGUMENT VALIDATIONS
        //Prevent 0 amount or dust attacks (500 lamports)
        require!(args.deposit_lamports > 500, OridionError::InvalidDepositAmount);

        //1 Delay, 2 Instant, 3 Manual
        require!(args.mode <= 3, OridionError::InvalidMode);

        // Check: length must be exactly 6 characters
        // Basic validity checks
        require!(nonzero_32(&args.passcode), OridionError::InvalidPasscode);

        let clock: Clock = Clock::get()?;
        let now = clock.unix_timestamp;

        //Set the landing timestamp from delay.
        //We no longer pass the landing timestamp because of timezone/vpn client side issues.
        let land_at = now + args.delay as i64;

        // If delay mode, require that the landing time is after now.
        if args.mode == 1 {
            require!(land_at > now,OridionError::InvalidLandingTimestamp);
        }

        // Delay cannot be more than 24 hours for now.
        require!(args.delay <= MAX_DELAY_ALLOWED, OridionError::InvalidDelay);
        // -------------------------------------------------//

        // -------------------------------------------------//
        // POD VALUES VALIDATION
        // Ensure the account is not already in use.
        let pod: &mut Account<Pod> = &mut ctx.accounts.pod;
        require!(
            pod.version == 0 &&
            pod.destination == Pubkey::default(),
            OridionError::ExistingPodActive
        );

        // Recombine into 32 bytes
        let mut dest_bytes = [0u8; 32];
        dest_bytes[..16].copy_from_slice(&args.d0);
        dest_bytes[16..].copy_from_slice(&args.d1);

        // Convert to Pubkey and store
        let dest_pk = Pubkey::new_from_array(dest_bytes);
        require!(dest_pk != Pubkey::default(), OridionError::InvalidDestination);
        // -------------------------------------------------//


        // -------------------------------------------------//
        // FEE CALCULATIONS
        let universe_account: &Account<Universe> = &ctx.accounts.universe;
        let hops = (args.delay as u64) / 180;
        let required_fee = match args.mode {
            1 => universe_account.fee + (hops * universe_account.increment), // Delay
            _ => universe_account.fee,  // Instant
        };
        let total_payment = args.deposit_lamports + required_fee;

        // Validate the creator's funds to cover the total cost
        let creator_balance = ctx.accounts.creator.lamports();
        require!(
            creator_balance >= total_payment,
            OridionError::InsufficientFunds
        );
        // -------------------------------------------------//


        // -------------------------------------------------//
        //  TRANSPARENCY LOG
        let creator_str = ctx.accounts.creator.key().to_string();
        let short_creator = if creator_str.len() > 9 {
            format!(
                "{}...{}",
                &creator_str[..5],
                &creator_str[creator_str.len() - 4..]
            )
        } else {
            creator_str.clone()
        };
        msg!(
            "Oridion: Transferring {} lamports from {} to Universe PDA",
            args.deposit_lamports,
            short_creator
        );
        // -------------------------------------------------//


        // -------------------------------------------------//
        // OPTIONAL SPL MEMO
        // If the user has opted in to transparency, add an SPL memo to help wallets
        // and explorers identify the transaction. This is optional and user-controlled.
        if args.show_memo == 1 {
            let creator_str = ctx.accounts.creator.key().to_string();
            let short_creator = if creator_str.len() > 9 {
                format!("{}...{}", &creator_str[..5], &creator_str[creator_str.len() - 4..])
            } else {
                creator_str
            };
            let memo = format!("Oridion: {} pod from {}", mode_string(args.mode), short_creator);
            let ix = spl_memo::build_memo(memo.as_bytes(), &[]);
            invoke(&ix, &[ctx.accounts.creator.to_account_info()])?;
        }

        // -------------------------------------------------//


        // -------------------------------------------------//
        // POD META
        let pod_meta = &mut ctx.accounts.pod_meta;
        // Init user meta if isn't initialized already
        if pod_meta.created_at == 0 {
            pod_meta.authority = *ctx.accounts.creator.key;
            pod_meta.created_at = Clock::get()?.unix_timestamp;
        }
        //If pod id is not already found, add it. Prune if over max.
        if !pod_meta.ids.contains(&id) {
            pod_meta.ids.push(id);
            pod_meta.prune_ids();
        }
        // -------------------------------------------------//


        // -------------------------------------------------//
        // POD & FEE TRANSFER
        let transfer_deposit: Instruction = transfer(
            ctx.accounts.creator.key, // Convert Pubkey to bytes and then to Address
            universe_account.to_account_info().key, // Same process for universe_account
            args.deposit_lamports,
        );

        invoke_signed(
            &transfer_deposit,
            &[
                ctx.accounts.creator.to_account_info(),
                universe_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;

        let fee_transfer_ix: Instruction = transfer(
            ctx.accounts.creator.key,
            ctx.accounts.treasury.key,
            required_fee,
        );
        invoke_signed(
            &fee_transfer_ix,
            &[
                ctx.accounts.creator.to_account_info(),
                ctx.accounts.treasury.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;
        // -------------------------------------------------//


        // -------------------------------------------------//
        // Save pod data
        pod.account_type = AccountType::Pod as u8;
        pod.version = 1;
        pod.mode = args.mode;
        pod.id = id;
        pod.created_at = now;
        pod.last_process_at = now;
        pod.hops = 1;
        pod.lamports = args.deposit_lamports;
        pod.last_process = 0; //0 = launch pod
        pod.delay = args.delay;
        pod.land_at = land_at;
        pod.passcode_hash = args.passcode; // pass code hash: [u8; 32]
        pod.is_in_transit = 0;
        pod.destination = dest_pk;
        pod.location = ctx.accounts.planet.key();

        //Depending on the land_at timestamp, set the next process and hop process timestamp
        //If land_at is less than 3 minutes away (180), land is the next process. Else, hop.
        if args.mode == 3 {
            pod.next_process = 0; //0 always
            pod.next_process_at = 0 // always;
        } else if (now + 180) > land_at {
            //Landing is next.
            pod.next_process = 1; //1 = land
            pod.next_process_at = land_at;
        } else {
            //Hop is next. Set the next hop processing timestamp
            pod.next_process = 0; //0 = hop
            pod.next_process_at = now + 180;
        }

        // -------------------------------------------------//
        // Planet bookkeeping - Increment visits
        let planet = &mut ctx.accounts.planet;
        planet.visits += 1;

        // TRANSACTION - From galaxy to planet
        //msg!("Hopping from galaxy to {}", planet_account.name);
        planet.add_lamports(args.deposit_lamports)?;
        ctx.accounts.universe.sub_lamports(args.deposit_lamports)?;

        //Initialize Activity log PDA - if not instant
        if pod.mode != 2 {
            pod.init_log(&planet.name, now);
        }

        msg!(
            "Pod launched successfully: type={}, from {}",
            mode_string(args.mode),
            short_creator
        );
        Ok(())
    }


    /// LOCK PLANET - Locks the planet during transaction
    // If unlocked, lock it - (locked_at == 0 -> Proceeds and sets new lock)
    // If already locked (locked != 0) but lock time has expired (more than LOCK_EXPIRED_SECONDS),
    // then clear the lock and relock it. (locked_at != 0 && expired - Clears old lock, sets new one)
    // If already locked (locked != 0) but has not expired yet, then fail.
    // Locked_at != 0 && has not expired - Fails with PlanetStillLocked
    pub fn lock_node(ctx: Context<LockPlanet>) -> Result<()> {
        let planet = &mut ctx.accounts.planet;
        let pod = &ctx.accounts.pod;
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;
    
        let is_locked = planet.locked_at != 0;
        let lock_age = now - planet.locked_at;
        let is_lock_expired = lock_age > LOCK_EXPIRE_SECONDS;
    
        // If the planet is locked, it must be expired to proceed
        if is_locked {
            require!(is_lock_expired, OridionError::PlanetStillLocked);
        }
    
        // Set to lock
        planet.locked_by = pod.key();
        planet.locked_at = now;
    
        Ok(())
    }


    /// SCATTER LOCK - Same lock but for scatter hop (3 planets)
    pub fn fan_lock(ctx: Context<ScatterLockPlanets>) -> Result<()> {
        let pod = &ctx.accounts.pod;
        let now = Clock::get()?.unix_timestamp;

        let planets = [
            &mut ctx.accounts.planet_1,
            &mut ctx.accounts.planet_2,
            &mut ctx.accounts.planet_3,
        ];

        for planet in planets {
            let is_locked = planet.locked_at != 0;
            let lock_age = now - planet.locked_at;
            let is_lock_expired = lock_age > LOCK_EXPIRE_SECONDS;

            // If the planet is locked, it must be expired to proceed
            if is_locked {
                require!(is_lock_expired, OridionError::PlanetStillLocked);
            }

            // Set lock
            planet.locked_by = pod.key();
            planet.locked_at = now;
        }
        Ok(())
    }



    /// HOP FROM PLANET TO PLANET
    pub fn route1(ctx: Context<PlanetHop>) -> Result<()>{

        let pod: &mut Account<Pod> = &mut ctx.accounts.pod;
        let from: &mut Account<Planet> = &mut ctx.accounts.from_planet;
        let to: &mut Account<Planet> = &mut ctx.accounts.to_planet;
        let book: &mut Account<LandBook> = &mut ctx.accounts.book;

        // Check planet is unlocked
        validate_planet_is_usable(from,pod.key())?;

        // Validate `from` and `to` planets are different
        require!(
            from.name != to.name,
            OridionError::HopErrorToAndFromAreSame
        );

        // Validate sufficient funds in the `from_planet` account
        require!(
            from.get_lamports() >= pod.lamports,
            OridionError::InsufficientFunds
        );

        // Update pod with new data
        pod.location = to.key();
        hop_pod(pod, book)?;

        //Increment visits
        to.visits += 1;

        //Log activity - (not instant)
        if pod.mode != 2 {
            pod.log_activity(ActivityAction::Hop as u8, &to.name)?;
        }

        // TRANSACTION: Move funds from planet to planet
        to.add_lamports(pod.lamports)?;
        from.sub_lamports(pod.lamports)?;

        //Release planet lock
        release_planet_lock(from)?;
        Ok(())
    }



    /// STAR HOP TWO - START (PLANET -> STAR1|START2)
    /// The "from" planet must be locked before.
    pub fn route2_start(
        ctx: Context<StarHopTwoStart>,
        _star_one_id: String,
        _star_two_id: String
    ) -> Result<()>{

        let pod: &mut Account<Pod> = &mut ctx.accounts.pod;

        let manager: &Signer = &ctx.accounts.manager;
        let star1: &mut Account<Star> = &mut ctx.accounts.star_one;
        let star2: &mut Account<Star> = &mut ctx.accounts.star_two;
        let from: &mut Account<Planet> = &mut ctx.accounts.from_planet;


        // IMPORTANT VALIDATION:
        // Planet must be usable. Stars cannot be the same. Pod cannot be in transit.
        validate_planet_is_usable(from,pod.key())?;
        require!(pod.is_in_transit == 0, OridionError::InTransit);
        require!(star1.key() != star2.key(), OridionError::HopErrorStarsMustBeUnique);

        //Set immediately after validations
        pod.is_in_transit = 1;
        pod.last_process_at = Clock::get()?.unix_timestamp;

        let percent: u8 = get_random_percent();
        let star_one_amount: u64 = (percent as u64 * pod.lamports) / 100;
        let star_two_amount: u64 = pod.lamports - star_one_amount;
        //msg!("Hopping to star 1: {}", star_one_amount.to_string());
        //msg!("Hopping to Star 2: {}", star_two_amount.to_string());

        //Make sure the amounts are equal to the pod accounts set lamports amount
        require!(star_one_amount + star_two_amount == pod.lamports, OridionError::StarHopCalculationError);

        //Set star key here - must do to prevent errors
        let star1_key = star1.key();
        let star2_key = star2.key();

        //Set amounts to accounts
        star1.amount = star_one_amount;
        star2.amount = star_two_amount;
        star1.manager = *manager.key;
        star2.manager = *manager.key;

        // TRANSACTION - Transfer from planet to star one and two
        ctx.accounts.star_one.add_lamports(star_one_amount)?;
        ctx.accounts.star_two.add_lamports(star_two_amount)?;
        from.sub_lamports(pod.lamports)?;

        // STORE META
        let meta = &mut ctx.accounts.star_meta;
        meta.to_pdas = [star1_key, star2_key, Pubkey::default(), ];
        meta.bump = ctx.bumps.star_meta;

        //Release planet lock
        release_planet_lock(from)?;
        Ok(())
    }



    /// STAR HOP TWO - END - (STAR1|START2 -> PLANET)
    pub fn route2_end(ctx: Context<StarHopTwoEnd>) -> Result<()>{

        let pod: &mut Account<Pod> = &mut ctx.accounts.pod;
        let book: &mut Account<LandBook> = &mut ctx.accounts.book;

        require!(pod.is_in_transit == 1, OridionError::NotInTransit);
        pod.is_in_transit = 0;

        let to: &mut Account<Planet> = &mut ctx.accounts.to_planet;
        //let to_planet_name: String = to.name.to_owned();
        let star1: &mut Account<Star> = &mut ctx.accounts.star_one;
        let star2: &mut Account<Star> = &mut ctx.accounts.star_two;
        let star_one_amount: u64 = star1.amount.clone();
        let star_two_amount: u64 = star2.amount.clone();

        require!(star_one_amount + star_two_amount == pod.lamports, OridionError::StarHopCalculationError);

        // Update pod with new data
        pod.location = to.key();
        hop_pod(pod,book)?;

         //Clear our star amount
        star1.amount = 0;
        star2.amount = 0;
     
        //Increment planet visit
        to.visits += 1;

        //Log activity - not instant
        if pod.mode != 2 {
            pod.log_activity(ActivityAction::Star2 as u8, &to.name)?;
        }

        // TRANSACTIONS
        // Transaction from stars one and two to the destination planet
        let total_lamports: u64 = star_one_amount + star_two_amount;
        ctx.accounts.star_one.sub_lamports(star_one_amount)?;
        ctx.accounts.star_two.sub_lamports(star_two_amount)?;
        ctx.accounts.to_planet.add_lamports(total_lamports)?;

        // EXPLODE STARS - Transfer out remaining lamports
        let star_one_remaining_lamports = ctx.accounts.star_one.get_lamports();
        let star_two_remaining_lamports = ctx.accounts.star_two.get_lamports();

        if star_one_remaining_lamports > 0 {
            ctx.accounts.manager.add_lamports(star_one_remaining_lamports)?;
            ctx.accounts.star_one.sub_lamports(star_one_remaining_lamports)?;
        }
        if star_two_remaining_lamports > 0 {
            ctx.accounts.manager.add_lamports(star_two_remaining_lamports)?;
            ctx.accounts.star_two.sub_lamports(star_two_remaining_lamports)?;
        }

        Ok(())
    }


    /// STAR HOP THREE - START (PLANET -> STAR1|START2|STAR3)
    pub fn route3_start(
        ctx: Context<StarHopThreeStart>,
        _star_one_id: String,
        _star_two_id: String,
        _star_three_id: String
    ) -> Result<()>{

        let pod: &mut Account<Pod> = &mut ctx.accounts.pod;
        let from: &mut Account<Planet> = &mut ctx.accounts.from_planet;
        let manager: &Signer = &ctx.accounts.manager;

        let star1: &mut Account<Star> = &mut ctx.accounts.star_one;
        let star2: &mut Account<Star> = &mut ctx.accounts.star_two;
        let star3: &mut Account<Star> = &mut ctx.accounts.star_three;

        // IMPORTANT VALIDATION:
        // Validate the planet is locked. Planet cannot be in transit. Stars cannot be the same
        validate_planet_locked_by_pod(from,pod.key())?;
        require!(pod.is_in_transit == 0, OridionError::InTransit);
        require!(star1.key() != star2.key(), OridionError::HopErrorStarsMustBeUnique);
        require!(star2.key() != star3.key(), OridionError::HopErrorStarsMustBeUnique);
        require!(star1.key() != star3.key(), OridionError::HopErrorStarsMustBeUnique);

        // Set in transit
        pod.is_in_transit = 1;
        pod.last_process_at = Clock::get()?.unix_timestamp;


        let (first_split_percent, second_split_percent) = (get_random_percent(), get_random_percent());
        let side_one = (first_split_percent as u64 * pod.lamports) / 100;
        let side_two = pod.lamports - side_one;

        let (mut star_one_amount, star_two_amount, star_three_amount) =
            if side_one > side_two {
                let one = (second_split_percent as u64 * side_one) / 100;
                let three = side_one - one;
                (one, side_two, three)
            } else {
                let two = (second_split_percent as u64 * side_two) / 100;
                let three = side_two - two;
                (side_one, two, three)
            };

        // Final correction (avoid rounding mismatch)
        let computed_total = star_one_amount + star_two_amount + star_three_amount;

        // Dust Correction:
        // Due to integer division rounding during random percent splits,
        // it's possible that the final sum of the star split amounts
        // is slightly less than the original deposit amount (i.e., some "dust" is lost).
        //
        // We fix this by calculating the difference and adding the missing dust
        // back into `star_one_amount`. We use `.saturating_add()` to safely avoid
        // any potential overflow (even though unlikely).
        //
        // This guarantees that the total amount sent to stars always equals
        // the original deposit amount, preserving value integrity.
        let diff = pod.lamports.checked_sub(computed_total).unwrap_or(0);
        star_one_amount = star_one_amount.saturating_add(diff); // Dust absorbed by star_one

        // Final sanity check
        require!(
            star_one_amount + star_two_amount + star_three_amount == pod.lamports,
            OridionError::StarHopCalculationError
        );

        let star1_key = star1.key();
        let star2_key = star2.key();
        let star3_key = star3.key();

        star1.amount = star_one_amount;
        star2.amount = star_two_amount;
        star3.amount = star_three_amount;
        star1.manager = *manager.key;
        star2.manager = *manager.key;
        star3.manager = *manager.key;

        // Transfer from planet to star one and two
        ctx.accounts.star_one.add_lamports(star_one_amount)?;
        ctx.accounts.star_two.add_lamports(star_two_amount)?;
        ctx.accounts.star_three.add_lamports(star_three_amount)?;
        from.sub_lamports(pod.lamports)?;

        let rent = Rent::get()?;
        let space = 8 + StarMeta::INIT_SPACE;
        let lamports = rent.minimum_balance(space);

        let pod_key = pod.to_account_info().key;
        let seeds = &[b"star_three", pod_key.as_ref()];
        let (pda, bump) = Pubkey::find_program_address(seeds, ctx.program_id);
        let signer_seeds = &[b"star_three", pod_key.as_ref(), &[bump]];

        //Get star meta
        let star_meta = &mut ctx.accounts.star_meta;

        //star_meta account passed must be the same as the derived here.
        require!(star_meta.key() == pda,OridionError::InvalidStarMetaPda);

        // Safety check: Make sure the PDA isn't already initialized
        require!(
            star_meta.to_account_info().lamports() == 0,
            OridionError::PdaAlreadyInitialized
        );

        // Create the star_meta data account
        let system_program = &ctx.accounts.system_program;
        invoke_signed(
            &create_account(
                manager.key,
                &pda,
                lamports,
                space as u64,
                ctx.program_id,
            ),
            &[
                manager.to_account_info(),
                star_meta.to_account_info(),
                system_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;


        // Set the star_meta data
        let meta = StarMeta {
            to_pdas: [star1_key, star2_key, star3_key],
            bump,
        };

        // OLD WAY
        //let mut data = star_meta.try_borrow_mut_data()?;
        // Write discriminator first
        // let discriminator = StarMeta::discriminator();
        // data[..8].copy_from_slice(&discriminator);
        // Then serialize the struct starting after the discriminator
        //meta.serialize(&mut &mut data[8..])?;

        let mut data = star_meta.try_borrow_mut_data()?;
        meta.try_serialize(&mut &mut data[..])?;

        //Release lock
        release_planet_lock(from)?;

        Ok(())
    }


    /// STAR HOP THREE - END (STAR1|START2|STAR3 -> PLANET)
    pub fn route3_end(ctx: Context<StarHopThreeEnd>) -> Result<()>{

        let pod: &mut Account<Pod> = &mut ctx.accounts.pod;
        let book: &mut Account<LandBook> = &mut ctx.accounts.book;


        require!(pod.is_in_transit == 1, OridionError::NotInTransit);
        pod.is_in_transit = 0;

        let to: &mut Account<Planet> = &mut ctx.accounts.to_planet;
        let star1: &mut Account<Star> = &mut ctx.accounts.star_one;
        let star2: &mut Account<Star> = &mut ctx.accounts.star_two;
        let star3: &mut Account<Star> = &mut ctx.accounts.star_three;
        let star_one_amount: u64 = star1.amount.clone();
        let star_two_amount: u64 = star2.amount.clone();
        let star_three_amount: u64 = star3.amount.clone();

        require!(star_one_amount + star_two_amount + star_three_amount == pod.lamports, OridionError::StarHopCalculationError);

        // Update pod location
        pod.location = to.key();
        hop_pod(pod,book)?;

         //Clear our star amount
        star1.amount = 0;
        star2.amount = 0;
        star3.amount = 0;
     
        //Increment planet visit
        to.visits += 1;

        //Log activity - not instant
        if pod.mode != 2 {
            pod.log_activity(ActivityAction::Star3 as u8, &to.name)?;
        }

        // TRANSACTIONS
        // Transaction from stars one and two to the destination planet
        ctx.accounts.star_one.sub_lamports(star_one_amount)?;
        ctx.accounts.star_two.sub_lamports(star_two_amount)?;
        ctx.accounts.star_three.sub_lamports(star_three_amount)?;
        ctx.accounts.to_planet.add_lamports(pod.lamports)?;

        // EXPLODE STARS - Transfer out remaining lamports
        // Transfer remaining lamports (dust, rent) from star accounts after transfer.
        // This ensures complete cleanup and consolidates unused funds back to the manager.
        for star in [&ctx.accounts.star_one, &ctx.accounts.star_two, &ctx.accounts.star_three] {
            let remaining = star.get_lamports();
            ctx.accounts.manager.add_lamports(remaining)?;
            star.sub_lamports(remaining)?;
        }
        Ok(())
    }


    /// SCATTER HOP - START (PLANET -> PLANET1|PLANET2|PLANET3|PLANET4|PLANET5)
    pub fn fan_start(ctx: Context<ScatterStart>) -> Result<()> {
        let pod = &mut ctx.accounts.pod;
        let from = &mut ctx.accounts.from_planet;

        // Validate that the "from" planet is locked or pod owns the lock
        validate_planet_locked_by_pod(from, pod.key())?;

        require!(pod.is_in_transit == 0, OridionError::InTransit);

        let total = pod.lamports;
        require!(total > 0, OridionError::InvalidDepositAmount);
        require!(from.get_lamports() >= total, OridionError::PlanetNotEnoughFundsError);

        // Generate pseudo-random split using block timestamp
        let now = Clock::get()?.unix_timestamp;
        let rng = anchor_lang::solana_program::keccak::hashv(&[&now.to_le_bytes()]);
        let mut splits = [0u64; 3];
        let mut remaining = total;

        for i in 0..2 {
            let rand_seed = u64::from_le_bytes([
                rng.0[i], rng.0[i + 1], rng.0[i + 2], rng.0[i + 3],
                rng.0[i + 4], rng.0[i + 5], rng.0[i + 6], rng.0[i + 7],
            ]);
            let rand_percent = (rand_seed % 50) + 1;
            let amt = (total * rand_percent) / 100;
            splits[i] = amt.min(remaining);
            remaining -= splits[i];
        }

        splits[2] = remaining;

        require!(
        splits.iter().sum::<u64>() == total,
        OridionError::ScatterSplitMathError
    );

        let planets = [
            &mut ctx.accounts.to_planet_1,
            &mut ctx.accounts.to_planet_2,
            &mut ctx.accounts.to_planet_3,
        ];

        // Subtract from source
        from.sub_lamports(total)?;

        // Distribute to destination planets
        for i in 0..3 {
            planets[i].add_lamports(splits[i])?;
        }

        // Store meta data
        let meta = &mut ctx.accounts.scatter_meta;
        meta.from = from.key();
        meta.to_pdas = [
            planets[0].key(),
            planets[1].key(),
            planets[2].key()
        ];
        meta.amounts = [
            splits[0],
            splits[1],
            splits[2]
        ];
        meta.created_at = now;
        meta.bump = ctx.bumps.scatter_meta;

        pod.is_in_transit = 1;
        pod.last_process_at = now;

        release_planet_lock(from)?;

        Ok(())
    }


    /// SCATTER HOP - END (PLANET1|PLANET2|PLANET3|PLANET4|PLANET5 -> PLANET)
    pub fn fan_end(ctx: Context<ScatterEnd>) -> Result<()> {
        let meta = &ctx.accounts.scatter_meta;
        let pod = &mut ctx.accounts.pod;
        let to = &mut ctx.accounts.to_planet;
        let book = &mut ctx.accounts.book;

        //Validate transmit meta is found and the pod is in transit.
        require!(meta.created_at > 0,OridionError::InvalidScatterMeta);
        require!(pod.is_in_transit == 1, OridionError::NotInTransit);

        //Gather all planets to transfer from
        let from_planets = [
            &mut ctx.accounts.from_planet_1,
            &mut ctx.accounts.from_planet_2,
            &mut ctx.accounts.from_planet_3,
        ];

        //Validate all planets are usable.
        for i in 0..3 {
            validate_planet_is_usable(from_planets[i],pod.key())?;
        }


        //Tally the total
        let mut total_collected: u64 = 0;
        for i in 0..3 {
            let amt = meta.amounts[i];
            require!(
                from_planets[i].get_lamports() >= amt,
                OridionError::PlanetNotEnoughFundsError
            );
            total_collected = total_collected
                .checked_add(amt)
                .ok_or(OridionError::UnusualMathError)?;

            //Release echo planet lock
            release_planet_lock(from_planets[i])?;
        }

        // Check that total matches original pod deposit
        require!(
            total_collected == pod.lamports,
            OridionError::ScatterTotalToPodMismatch
        );

        // Update pod location
        pod.location = to.key();
        hop_pod(pod,book)?;

        //Log activity - not instant
        if pod.mode != 2 {
            pod.log_activity(ActivityAction::Scatter as u8, &to.name)?;
        }

        // Perform transfers
        for i in 0..3 {
            let amt = meta.amounts[i];
            from_planets[i].sub_lamports(amt)?;
            to.add_lamports(amt)?;
        }

        pod.is_in_transit = 0;

        Ok(())
    }


    /// LAND FUNDS TO FINAL DESTINATION.
    /// - Handles the transaction from the pod's current planet to set final destination user wallet.
    /// - This is just like planet hop except deliver to destination wallet
    /// Since we MUST pass the destination wallet address in the accounts section,
    /// and it must match the set `pod.destination` already; there is no need for further destination checks here.
    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct LandArgs {
        pub id: u16,
        pub created_at: i64,
        pub lamports: u64,
    }
    pub fn settle(ctx: Context<LandAccount>, args: LandArgs) -> Result<()> {
        let book = &mut ctx.accounts.book;
        let from: &mut Account<Planet> = &mut ctx.accounts.from_planet;
        let destination: &mut SystemAccount = &mut ctx.accounts.destination;

        let dest_key = destination.key();
        let expect = token_from(args.id, &dest_key, args.lamports, args.created_at);

        // 1) locate by token (small N; linear scan is fine)
        let Some(i) = book.tickets.iter().position(|t| *t == expect)
        else { return err!(OridionError::TicketNotFound) };

        // If token matches, then we can proceed with confidence that nothing has been tampered with.

        // Set delivery lamports
        let delivery_lamports = args.lamports;

        // 2) VALIDATION: Prevent lamports over spend
        require!(from.get_lamports() >= delivery_lamports,OridionError::PlanetNotEnoughFundsError);

        // 3) TRANSFER funds from Planet → Destination
        ctx.accounts.destination.add_lamports(delivery_lamports)?;
        from.sub_lamports(delivery_lamports)?;

        // 4) erase in O(1) without shifting
        book.tickets.swap_remove(i);

        //Release planet lock
        release_planet_lock(from)?;
        Ok(())
    }


    //Force land pod by signer
    pub fn usr_settle(ctx: Context<EmergencyLandByCreator>) -> Result<()> {
        let pod = &mut ctx.accounts.pod;
        let from_planet = &mut ctx.accounts.from_planet;
        let delivery_lamports = pod.lamports;

        let now = Clock::get()?.unix_timestamp;
        require!(
            now > pod.created_at + 180, // 3 minutes
            OridionError::TooSoonToEmLand
        );

        // Check planet is unlocked
        validate_planet_is_usable(from_planet, pod.key())?;

        // VALIDATION: Prevent a double-landing or underfunded source
        require!(delivery_lamports > 0, OridionError::AlreadyLanded);
        require!(
            from_planet.get_lamports() >= delivery_lamports,
            OridionError::PlanetNotEnoughFundsError
        );

        // Destinations must match
        require!(
            ctx.accounts.destination.key() == pod.destination,
            OridionError::InvalidDestination
        );

        // TRANSFER funds from Planet → Destination
        ctx.accounts.destination.add_lamports(delivery_lamports)?;
        from_planet.sub_lamports(delivery_lamports)?;

        release_planet_lock(from_planet)?;
        Ok(())
    }


    /// Emergency land with code
    pub fn code_settle(
        ctx: Context<EmergencyLandWithCode>,
        passcode: String, // user provides passcode as plain text
    ) -> Result<()> {
        let pod = &mut ctx.accounts.pod;
        let from_planet = &mut ctx.accounts.from_planet;

        // 1. Ensure the planet is usable and not locked by someone else
        validate_planet_is_usable(from_planet, pod.key())?;

        // 2. Prevent double-landing and check planet funding
        require!(pod.lamports > 0, OridionError::AlreadyLanded);
        require!(
            from_planet.get_lamports() >= pod.lamports,
            OridionError::PlanetNotEnoughFundsError
        );

        // 3. Validate passcode hash
        use anchor_lang::solana_program::hash::hash;
        let clean_passcode = passcode.trim().to_ascii_uppercase();
        let hash = hash(clean_passcode.as_bytes());
        require!(
            hash.to_bytes() == pod.passcode_hash,
            OridionError::InvalidPasscode
        );

        // 4. Destination safety check
        require!(
            ctx.accounts.destination.key() == pod.destination,
            OridionError::InvalidDestination
        );

        // 5. Transfer lamports from planet to destination
        let delivery_lamports = pod.lamports;
        ctx.accounts.destination.add_lamports(delivery_lamports)?;
        from_planet.sub_lamports(delivery_lamports)?;

        // 6. Release lock
        release_planet_lock(from_planet)?;

        Ok(())
    }

    //Close pod
    pub fn reclaim(_ctx: Context<ClosePod>) -> Result<()> {
        Ok(())
    }
}