// use anchor_lang::prelude::*;
// use anchor_lang::context::Context;
// use crate::accounts_planet::{BalancePlanets,Planet};
// use crate::errors::OridionError;


// pub fn fix_discriminator(ctx: Context<FixStarMeta>) -> Result<()> {
//     let discriminator = StarMeta::discriminator();
//     let mut data = ctx.accounts.star_meta.try_borrow_mut_data()?;
//     data[..8].copy_from_slice(&discriminator);
//     Ok(())
// }

// pub fn em_land(ctx: Context<EmLand>, amount: u64) -> Result<()> {
//     let planet: &mut Account<Planet> = &mut ctx.accounts.planet;
//     ctx.accounts.destination.add_lamports(amount)?;
//     planet.sub_lamports(amount)?;
//     Ok(())
// }


///-------------------------------------------------------------------///
/// REBALANCE PLANETS
/// Remove once stable
/// -------------------------------------------------------------------///
// pub fn rebalance_planet(ctx: Context<BalancePlanets>, amount_lamports: u64) -> Result<()>{
// 
//     //Balance from/to
//     let from: &mut Account<Planet> = &mut ctx.accounts.from_planet;
//     let to: &mut Account<Planet> = &mut ctx.accounts.to_planet;
// 
//     // --- SECURITY CHECK: Validate `from` and `to` planets are different ---
//     require!(
//             from.name != to.name,
//             OridionError::HopErrorToAndFromAreSame
//     );
// 
//     // --- SECURITY CHECK: Validate sufficient funds in the `from_planet` account ---
//     require!(
//             from.get_lamports() >= amount_lamports,
//             OridionError::InsufficientFunds
//         );
// 
//     // TRANSACTION: Move funds from planet to planet
//     ctx.accounts.to_planet.add_lamports(amount_lamports)?;
//     ctx.accounts.from_planet.sub_lamports(amount_lamports)?;
//     Ok(())
// }

// ///-------------------------------------------------------------------///
// /// DELETE UNIVERSE
// ///-------------------------------------------------------------------///
// pub fn delete_universe(ctx: Context<DeleteUniverse>) -> Result<()> {
//
//     //Remove planet from universe list
//     let universe: &mut Account<Universe> = &mut ctx.accounts.universe;
//     let planet: &mut Account<Planet> = &mut ctx.accounts.planet;
//     universe.p.retain(|x| x != &planet.name);
//     //msg!("== PLANET {} DELETED ==", planet.name.to_string());
//     //msg!("== PLANET DELETED ==");
//     Ok(())
// }


// //Emergency extract lamports from planet
// pub fn withdraw_orbit(ctx: Context<RetractOrbit>, deposit_lamports: u64 ) -> Result<()> {
//
//     let planet: &mut Account<Planet> = &mut ctx.accounts.planet;
//
//     //VALIDATION
//     let current_from_lamports_balance: u64 = planet.get_lamports();
//     require!(current_from_lamports_balance > deposit_lamports, OridionError::PlanetNotEnoughFundsError);
//
//     // TRANSACTION - Transfer to destination
//     ctx.accounts.destination.add_lamports(deposit_lamports)?;
//     ctx.accounts.planet.sub_lamports(deposit_lamports)?;
//
//     Ok(())
// }


//Helper function to convert planet names to [u8; 6]
// fn str_to_fixed_bytes(s: &str) -> [u8; 10] {
//     let mut buffer = [0u8; 10];
//     let bytes = s.as_bytes();
//     let len = bytes.len().min(10);
//     buffer[..len].copy_from_slice(&bytes[..len]);
//     buffer
// }

// Decode function kept for the future if needed.
// Fn u8_6_to_string(data: [u8; 6]) -> String {
//     let trimmed = data.iter().take_while(|&&b| b != 0).cloned().collect::<Vec<u8>>();
//     String::from_utf8(trimmed).unwrap_or_default()
// }

// pub fn get_planet_program_address(planet_name: &String, program_id: &Pubkey) -> Pubkey {
//     let(pk, _pda_bump) = Pubkey::find_program_address(&[
//         PLANET_PDA_SEED_PRE,
//         planet_name.as_ref(),
//         PLANET_PDA_SEED_POST
//     ], program_id);
//     pk
// }


// //TEMP DELETE
// pub fn unsafe_delete_planet(_ctx: Context<UnsafeDelete>) -> Result<()> {
//     Ok(())
// }

// #[derive(Accounts)]
// pub struct FixStarMeta<'info> {
//     #[account(mut)]
//     /// CHECK: This account is manually created with `invoke_signed` using a verified PDA.
//     pub star_meta: AccountInfo<'info>,
// }

// #[derive(Accounts)]
// pub struct EmLand<'info> {
//     #[account(mut, close = destination)] // No seeds used here
//     pub planet: Account<'info, Planet>,
//     #[account(mut)]
//     pub destination: SystemAccount<'info>,
//     #[account(mut, address = MANAGER_PUBKEY)]
//     pub manager: Signer<'info>
// }


// #[derive(Accounts)]
// pub struct UnsafeDelete<'info> {
//     #[account(mut, close = recipient)] // No seeds used here
//     pub target: Account<'info, Pod>,
//     #[account(mut)]
//     pub recipient: Signer<'info>,
// }

// #[derive(Accounts)]
// #[instruction(amount_lamports: u64)]
// pub struct EmFix<'info> {
//     #[account(mut)]
//     pub from_planet: Account<'info,Planet>,
//     #[account(mut, address = MANAGER_PUBKEY)]
//     pub manager: Signer<'info>
// }


// Em fix
// pub fn em_fix(ctx: Context<EmFix>,lamports: u64) -> Result<()> {
//     ctx.accounts.manager.add_lamports(lamports)?;
//     ctx.accounts.from_planet.sub_lamports(lamports)?;
//     Ok(())
// }