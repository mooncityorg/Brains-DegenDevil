use anchor_lang::prelude::*;
use std::mem::size_of;
mod utils;

pub use utils::{requestor_pda, vault_pda};
declare_id!("EChzoztcpa48HFuQwbFinb6f1JhtUTJjDXjCchP4GP9g");

const VAULT_PREFIX: &str = "DEGENRAND_VAULT_SEED_V1.0";
const REQUESTOR_PREFIX: &str = "DEGENRAND_REQUESTOR_SEED_V1.0";
#[program]
pub mod degenrand {

    use crate::{utils::remove_pda, ErrorCode};

    use super::*;

    const ORACLE_FEE: u64 = 495000; //  Approximately $0.09 = 0.000000001 * $175 * 495,000

    pub fn initialize(ctx: Context<Initialize>, request_bump: u8, vault_bump: u8) -> Result<()> {
        // Set the vault account, used to pay the oracle
        ctx.accounts.vault.requester = *ctx.accounts.requester.to_account_info().key;
        ctx.accounts.vault.bump = vault_bump;

        let requester = &mut ctx.accounts.requester.load_init()?;
        let clock: Clock = Clock::get().unwrap();

        // The requester is ZeroCopy and stores the random number
        requester.authority = *ctx.accounts.authority.key;
        requester.oracle = *ctx.accounts.oracle.key;
        requester.created_at = clock.unix_timestamp;
        requester.count = 0;
        requester.active_request = false;
        requester.last_updated = clock.unix_timestamp;
        requester.bump = request_bump;

        Ok(())
    }

    pub fn request_random(ctx: Context<RequestRandom>) -> Result<()> {
        // Some checks to ensure proper account ownership
        {
            let requester_key = ctx.accounts.requester.to_account_info().key();

            if requester_key != ctx.accounts.vault.requester {
                return Err(ErrorCode::Unauthorized.into());
            }

            let requester = &mut ctx.accounts.requester.load()?;
            let authority = ctx.accounts.authority.key();

            if requester.authority != authority {
                return Err(ErrorCode::Unauthorized.into());
            }

            if requester.oracle != ctx.accounts.oracle.key() {
                return Err(ErrorCode::WrongOracle.into());
            }

            if requester.active_request {
                return Err(ErrorCode::Unauthorized.into());
            }
        }

        // Transfer fee to Oracle
        {
            let vault = ctx.accounts.vault.to_account_info();

            **vault.try_borrow_mut_lamports()? = vault
                .lamports()
                .checked_sub(ORACLE_FEE)
                .ok_or(ProgramError::InvalidArgument)?;

            **ctx.accounts.oracle.try_borrow_mut_lamports()? = ctx
                .accounts
                .oracle
                .lamports()
                .checked_add(ORACLE_FEE)
                .ok_or(ProgramError::InvalidArgument)?;
        }

        // Once the requester has active_request set, it's frozen until the Oracle responds
        {
            let requester = &mut ctx.accounts.requester.load_mut()?;
            let clock: Clock = Clock::get().unwrap();

            requester.last_updated = clock.unix_timestamp;
            requester.active_request = true;
            requester.count += 1;
        }

        Ok(())
    }

    pub fn publish_random(
        ctx: Context<PublishRandom>,
        random: [u8; 64],
        pkt_id: [u8; 32],
        tls_id: [u8; 32],
    ) -> Result<()> {
        // Have to load the account this way to avoid automated ownership checks
        let loader: AccountLoader<Requester> =
            AccountLoader::try_from_unchecked(ctx.program_id, &ctx.remaining_accounts[0]).unwrap();
        let mut requester = loader.load_mut()?;

        if requester.oracle != ctx.accounts.oracle.key() {
            return Err(ErrorCode::Unauthorized.into());
        }

        if !requester.active_request {
            return Err(ErrorCode::AlreadyCompleted.into());
        }
        let clock: Clock = Clock::get().unwrap();

        requester.last_updated = clock.unix_timestamp;
        requester.active_request = false;
        requester.random = random;
        requester.pkt_id = pkt_id;
        requester.tls_id = tls_id;

        Ok(())
    }

    pub fn remove_pdas(ctx: Context<Cleanup>) -> Result<()> {
        let requester = ctx.accounts.requester.load()?;

        if requester.authority != ctx.accounts.initiator.key() {
            return Err(ErrorCode::Unauthorized.into());
        }

        let (vault_pda, _) = vault_pda(&ctx.accounts.initiator.key());

        if vault_pda != ctx.accounts.vault.key() {
            return Err(ErrorCode::Unauthorized.into());
        }

        remove_pda(
            &ctx.accounts.initiator.to_account_info(),
            &ctx.accounts.vault.to_account_info(),
        )?;

        remove_pda(
            &ctx.accounts.initiator.to_account_info(),
            &ctx.accounts.requester.to_account_info(),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [REQUESTOR_PREFIX.as_bytes(), authority.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + size_of::<Requester>()
    )]
    pub requester: AccountLoader<'info, Requester>,

    #[account(
        init,
        seeds = [VAULT_PREFIX.as_bytes(), authority.key().as_ref(), crate::id().as_ref()],
        bump,
        payer = authority,
        space = 8 + size_of::<Vault>()
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: The client decides the oracle to use
    #[account(mut)]
    pub oracle: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,
}

#[account(zero_copy)]
pub struct Requester {
    pub authority: Pubkey,
    pub oracle: Pubkey,
    pub created_at: i64,
    pub count: u64,
    pub last_updated: i64,
    pub random: [u8; 64],
    pub pkt_id: [u8; 32],
    pub tls_id: [u8; 32],
    pub active_request: bool,
    pub bump: u8,
}

#[account]
pub struct Vault {
    pub requester: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct RequestRandom<'info> {
    #[account(mut)]
    pub requester: AccountLoader<'info, Requester>,

    #[account(mut)]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: The client decides the oracle to use
    #[account(mut)]
    pub oracle: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PublishRandom<'info> {
    /// CHECK: oracle that will send random bytes.
    #[account(mut, signer)]
    pub oracle: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Cleanup<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Return Remaining amount to initiator
    #[account(mut)]
    pub requester: AccountLoader<'info, Requester>,

    /// CHECK: initiator to return amount to
    #[account(mut)]
    pub initiator: AccountInfo<'info>,
    /// CHECK: Return Remaining amount to initiator
    #[account(mut)]
    pub vault: Account<'info, Vault>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(mut)]
    pub requester: AccountLoader<'info, Requester>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: The client decides the new owner
    #[account(mut)]
    pub new_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to complete this transaction")]
    Unauthorized,

    #[msg("You have already completed this transaction")]
    AlreadyCompleted,

    #[msg("A request is already in progress. Only one request may be made at a time")]
    InflightRequest,

    #[msg("The Oracle you make the request with must be the same as initialization")]
    WrongOracle,

    #[msg("You cannot change authority of a request awaiting a response")]
    RequesterLocked,
}
