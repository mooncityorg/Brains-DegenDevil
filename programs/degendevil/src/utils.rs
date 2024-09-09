use std::{ops::DerefMut, str::FromStr};

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{Bet, DegenErrorCode, ADMIN_BET_PREFIX, VAULT_PREFIX};

/// Signer Seeds for Vault
///  let signer_seeds = &[
///     VAULT_PREFIX.as_bytes(),
///     mint.key.as_ref(),
///     program_id.as_ref(),
///     &[vault_bump],
/// ];
pub fn vault_pda(mint: &Pubkey, initiator: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            VAULT_PREFIX.as_bytes(),
            mint.as_ref(),
            initiator.as_ref(),
            &crate::id().as_ref(),
        ],
        &crate::id(),
    )
}

/// Handles the decimal value.
/// Converts to appropriate u64 representation.
pub fn calculate_amount(mint: &Mint, amount: u64) -> u64 {
    amount.saturating_mul(10_u64.pow((mint.decimals) as u32))
}

/// Signer Seeds for Coin
///  let signer_seeds =  &[
///    ADMIN_FEE_PREFIX.as_bytes(),
///    admin_account_pubkey()?.as_ref(),
///    &[admin_bet_bump],
/// ];
pub fn admin_bet_pda() -> Result<(Pubkey, u8)> {
    Ok(Pubkey::find_program_address(
        &[
            ADMIN_BET_PREFIX.as_bytes(),
            admin_account_pubkey()?.as_ref(),
        ],
        &crate::id(),
    ))
}

pub fn check_account_equals(key: &Pubkey, expected_key: &Pubkey) -> Result<()> {
    if expected_key.ne(key) {
        msg!("Account does not match the key.");
        return Err(DegenErrorCode::AccountMismatch.into());
    }

    Ok(())
}

pub fn remove_pda(creator: &AccountInfo, pda: &AccountInfo) -> Result<()> {
    **creator.try_borrow_mut_lamports()?.deref_mut() += pda.lamports();
    **pda.try_borrow_mut_lamports()?.deref_mut() = 0;
    Ok(())
}

// Bet token 52.5 A 75% chance win Token B
// Bet token 35 A 50% chance win Token B
// Bet 17.5 token A 25% chance win Token B
// Bet 7 token A 10% chance win Token B
pub fn calculate_probability(multiplier, random: &[u8]) -> u8 {
    let r50 = rand50(random[0]);
    let r75 = rand50(random[1]) | rand50(random[2]);
    let r90 = rand50(random[3]) | r75;
    let r5 = (1 - r90) & r50;

    match multiplier {
        133 => r75,
        200 => r50,
        400 => 1 - r75,
        1000 => 1 - r90,
        2000 => r5
        _ => random[4],
    }
}
pub fn rand50(rand: u8) -> u8 {
    &rand & 1
}

/// Bet Types
/// - finality: 700
/// - epoch: 1750
/// - cluster: 3500
/// - lamport: 5250
pub fn fee(bet: &Bet, amount: u64) -> u64 {
    match amount {
        amount if amount == bet.amount_lamport => bet.fee_finality,
        amount if amount == bet.amount_cluster => bet.fee_epoch,
        amount if amount == bet.amount_epoch => bet.fee_cluster,
        amount if amount == bet.amount_finality => bet.fee_lamport,
        _ => bet.fee_finality,
    }
}

const ADMIN_PUBKEY: &str = "7wqoNjMeCqg9ovoagZBsjcgkxKVLxQDXKADebEzviM5s";

pub fn admin_account_pubkey() -> Result<Pubkey> {
    Pubkey::from_str(ADMIN_PUBKEY).map_err(|_| DegenErrorCode::InvalidAdminPubkey.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn random_bytes() -> Vec<u8> {
        let mut rng = rand::thread_rng();

        (0..64).map(|_| rng.gen_range(0..=1)).collect()
    }

    #[test]
    fn test_probabilities() {
        let bet = Bet {
            amount_finality: 700,
            amount_epoch: 1750,
            amount_cluster: 3500,
            amount_lamport: 5250,
            ..Default::default()
        };
        let rand = random_bytes();

        let v = calculate_probability(&bet, 5250, rand.as_slice());
        println!("{}", v);

        let v = calculate_probability(&bet, 3500, rand.as_slice());
        println!("{}", v);

        let v = calculate_probability(&bet, 1750, rand.as_slice());
        println!("{}", v);

        let v = calculate_probability(&bet, 1000, rand.as_slice());
        println!("{}", v);

        let v = calculate_probability(&bet, 700, rand.as_slice());
        println!("{}", v);
    }

    #[test]
    fn test_probability_5250_amount() {
        let bet = Bet {
            amount_finality: 700,
            amount_epoch: 1750,
            amount_cluster: 3500,
            amount_lamport: 5250,
            ..Default::default()
        };
        let v = (0..=100).fold(0, |mut acc, _| {
            let rand = random_bytes();
            acc += calculate_probability(&bet, 5250, rand.as_slice());
            acc
        });

        println!("Amount : 5250, `{}` Wins per 100", v);
    }

    #[test]
    fn test_probability_3500_amount() {
        let bet = Bet {
            amount_finality: 700,
            amount_epoch: 1750,
            amount_cluster: 3500,
            amount_lamport: 5250,
            ..Default::default()
        };
        let v = (0..=100).fold(0, |mut acc, _| {
            let rand = random_bytes();
            acc += calculate_probability(&bet, 3500, rand.as_slice());
            acc
        });

        println!("Amount : 3500, `{}` Wins per 100", v);
    }

    #[test]
    fn test_probability_1750_amount() {
        let bet = Bet {
            amount_finality: 700,
            amount_epoch: 1750,
            amount_cluster: 3500,
            amount_lamport: 5250,
            ..Default::default()
        };
        let v = (0..=100).fold(0, |mut acc: u8, _| {
            let rand = random_bytes();
            acc += calculate_probability(&bet, 1750, rand.as_slice());
            acc
        });

        println!("Amount : 1750, `{}` Wins per 100", v);
    }

    #[test]
    fn test_probability_700_amount() {
        let bet = Bet {
            amount_finality: 700,
            amount_epoch: 1750,
            amount_cluster: 3500,
            amount_lamport: 5250,
            ..Default::default()
        };
        let v = (0..=100).fold(0, |mut acc: u8, _| {
            let rand = random_bytes();
            acc += calculate_probability(&bet, 700, rand.as_slice());
            acc
        });

        println!("Amount : 700, `{}` Wins per 100", v);
    }
}
