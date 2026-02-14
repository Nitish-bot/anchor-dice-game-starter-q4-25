use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use anchor_instruction_sysvar::Ed25519InstructionSignatures;

use crate::{errors::DiceError, state::Bet};

#[derive(Accounts)]
pub struct ResolveBet<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    ///CHECK: This is safe
    pub house: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"vault", house.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        mut,
        close = player,
        seeds = [b"bet", vault.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump = bet.bump
    )]
    pub bet: Account<'info, Bet>,
    #[account(
        address = anchor_lang::solana_program::sysvar::instructions::ID
    )]
    pub instruction_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    pub fn verify_ed25519_signature(&self, sig: &Vec<u8>) -> Result<()> {

        Ok(())
    }
    
    pub fn resolve_bet(&mut self, sig: &Vec<u8>, bumps: &ResolveBetBumps) -> Result<()> {
        Ok(())
    }
}