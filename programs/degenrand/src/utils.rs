use std::ops::DerefMut;

use anchor_lang::prelude::*;

use crate::{REQUESTOR_PREFIX, VAULT_PREFIX};

pub fn vault_pda(authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            VAULT_PREFIX.as_bytes(),
            authority.as_ref(),
            &crate::id().as_ref(),
        ],
        &crate::id(),
    )
}

/// Signer Seeds for Coin
///  let signer_seeds =  &[
///    COIN_PREFIX.as_bytes(),
///    initiator.key.as_ref(),
///    &[coin_bump],
/// ];
pub fn requestor_pda(authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[REQUESTOR_PREFIX.as_bytes(), authority.as_ref()],
        &crate::id(),
    )
}

pub fn remove_pda(creator: &AccountInfo, pda: &AccountInfo) -> Result<()> {
    **creator.try_borrow_mut_lamports()?.deref_mut() += pda.lamports();
    **pda.try_borrow_mut_lamports()?.deref_mut() = 0;
    Ok(())
}
