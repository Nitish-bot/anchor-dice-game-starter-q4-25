use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use anchor_instruction_sysvar::Ed25519InstructionSignatures;
use solana_program::{sysvar::instructions::{ID as SYSVAR_IX_ID, load_instruction_at_checked}, hash::hash, ed25519_program};

use crate::{errors::DiceError, state::Bet};

const HOUSE_EDGE: u128 = 150; // 1.5% House fees in basis points

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
        address = SYSVAR_IX_ID
    )]
    /// CHECK: Address checked
    pub instruction_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    pub fn verify_ed25519_signature(&self, sig: &Vec<u8>) -> Result<()> {
        let ix = load_instruction_at_checked(
            0,
            &self.instruction_sysvar.to_account_info()
        ).map_err(|_| DiceError::Ed25519Signature)?;

        require!(ix.program_id == ed25519_program::ID, DiceError::Ed25519Program);
        require!(ix.accounts.len() == 0, DiceError::Ed25519Accounts);

        let signatures = Ed25519InstructionSignatures::unpack(&ix.data).map_err(|_| DiceError::Ed25519Program)?.0;
        let signature = &signatures[0];
        
        require!(signatures.len() == 1, DiceError::Ed25519Signature);
        
        require!(signature.is_verifiable, DiceError::Ed25519Header);
        require_keys_eq!(signature.public_key.ok_or(DiceError::Ed25519Pubkey)?, self.house.key(), DiceError::Ed25519Pubkey);
        require!(&signature.signature.ok_or(DiceError::Ed25519Signature)?.eq(sig.as_slice()), DiceError::Ed25519Signature);

        require!(&signature.message.as_ref().ok_or(DiceError::Ed25519Signature)?.eq(&self.bet.to_slice()), DiceError::Ed25519Message);

        Ok(())
    }
    
    pub fn resolve_bet(&mut self, sig: &Vec<u8>, bumps: &ResolveBetBumps) -> Result<()> {
        let hash = hash(sig).to_bytes();
        let mut hash_16: [u8; 16] = [0; 16];

        hash_16.copy_from_slice(&hash[0..16]);
        let lower = u128::from_le_bytes(hash_16);
        
        hash_16.copy_from_slice(&hash[16..32]);
        let upper = u128::from_le_bytes(hash_16);

        let roll = lower
            .wrapping_add(upper)
            .wrapping_rem(100) as u8 + 1;

        if self.bet.roll > roll {
            let payout = (self.bet.amount as u128)
                .checked_mul(10000 - HOUSE_EDGE as u128).ok_or(DiceError::Overflow)?
                .checked_div(self.bet.roll as u128 - 1).ok_or(DiceError::Overflow)?
                .checked_div(100).ok_or(DiceError::Overflow)?;

            let signer_seeds: &[&[&[u8]]] = &[&[
                b"vault",
                &self.house.key().to_bytes(),
                &[bumps.vault],
            ]];

            let accounts = Transfer {
                from: self.vault.to_account_info(),
                to: self.player.to_account_info(),
            };

            let ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                accounts,
                signer_seeds
            );

            transfer(ctx, payout as u64)?;
        }

        Ok(())
    }
}