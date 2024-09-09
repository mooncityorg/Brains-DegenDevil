use anchor_lang::prelude::*;
use anchor_spl::token::{Approve, Mint, Token, TokenAccount, Transfer};
use std::mem::size_of;
mod utils;
pub use utils::*;

declare_id!("H2LCFgiKNFwdZyVQoJFhhhygvvuV8twbfzJ8nJpJHgG1");

const SOL_VAULT: &str = "SOL_VAULT";
const VAULT_PREFIX: &str = "DEGENDEVIL_VAULT_SEED_V1.0";
const ADMIN_BET_PREFIX: &str = "DEGENDEVIL_ADMIN_BET_V1.0";
const ORACLE_FEE: u64 = 1000000;

#[program]
pub mod degendevil {

    use super::*;

    pub fn create_coin(ctx: Context<CreateCoin>, multiplier: u64, amount: u64, sol_amount: u64) -> Result<()> {
        if ctx.accounts.admin.key() != admin_account_pubkey()? {
            return Err(DegenErrorCode::Unauthorized.into());
        }

        if sol_amount == 0 {

            if ctx.accounts.initiator_ata.amount < amount {
                return Err(DegenErrorCode::NotEnoughTokens.into());
            }

            anchor_spl::token::transfer(
                ctx.accounts.token_transfer_ctx(
                    ctx.accounts.initiator.to_account_info(),
                    ctx.accounts.initiator_ata.to_account_info(),
                    ctx.accounts.admin_ata.to_account_info(),
                ),
                amount,
            )?;
    
        } else {
            let ix = anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.initiator.key(),
                &ctx.accounts.sol_vault.key(),
                sol_amount,
            );
    
            anchor_lang::solana_program::program::invoke(
                &ix,
                &[
                    ctx.accounts.initiator.to_account_info(),
                    ctx.accounts.sol_vault.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
    
        }

        {
            let vault = &mut ctx.accounts.vault;

            vault.coin_info = CoinInfo {
                initiator: ctx.accounts.initiator.key(),
                amount,
                sol_amount,
                multiplier
            };
        }

        let mut oracle_fee = 0;
        match multiplier {
            133 => oracle_fee = sol_amount * 3/100,
            200 => oracle_fee = sol_amount * 3/100,
            400 => oracle_fee = sol_amount * 4/100,
            1000 => oracle_fee = sol_amount * 55/1000,
            2000 => oracle_fee = sol_amount * 65/1000,
            _ => oracle_fee = 0,
        }        
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.initiator.key(),
            &ctx.accounts.oracle_vault.key(),
            ORACLE_FEE + oracle_fee,
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.initiator.to_account_info(),
                ctx.accounts.oracle_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        let bet_fee = &mut ctx.accounts.bet;

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.initiator.key(),
            &ctx.accounts.admin.key(),
            fee(bet_fee, amount),
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.initiator.to_account_info(),
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        degenrand::cpi::request_random(ctx.accounts.request_random())?;

        Ok(())
    }

    pub fn reveal_coin<'key, 'accounts, 'remaining, 'info>(
        ctx: Context<'key, 'accounts, 'remaining, 'info, RevealCoin<'info>>,
        vault_bump: u8
    ) -> Result<()> {
        let initiator_pubkey = ctx.accounts.initiator.key();

        if ctx.accounts.vault.coin_info.initiator != initiator_pubkey {
            return Err(DegenErrorCode::Unauthorized.into());
        }

        let requester_loader: AccountLoader<degenrand::Requester> =
            AccountLoader::try_from_unchecked(ctx.program_id, &ctx.accounts.requester)?;

        let requester = requester_loader.load()?;

        if requester.active_request {
            return Err(DegenErrorCode::OracleNotCompleted.into());
        }

        let (bet_pda, admin_bet_bump) = admin_bet_pda()?;

        check_account_equals(&bet_pda, &ctx.accounts.bet.key())?;

        let admin_pubkey = admin_account_pubkey()?;

        let signer_seeds = &[
            ADMIN_BET_PREFIX.as_bytes(),
            admin_pubkey.as_ref(),
            &[admin_bet_bump],
        ];
        let mut sol_amount : u64 = 0;
        let mut amount : u64 = 0;

        // let cpi_accounts = Transfer {
        //     authority: ctx.accounts.bet.to_account_info(),
        //     from: ctx.accounts.admin_ata.to_account_info(),
        //     to: ctx.accounts.initiator_ata.to_account_info(),
        // };
        let multiplier = ctx.accounts.vault.coin_info.multiplier
        if  calculate_probability(multiplier, &requester.random,) == 1 {
            sol_amount = ctx.accounts.vault.coin_info.sol_amount * multiplier / 100;
            amount = ctx.accounts.vault.coin_info.amount * multiplier / 100;
        }

        let ix = spl_token::instruction::transfer(
            ctx.accounts.token_program.key,
            &ctx.accounts.admin_ata.key(),
            &ctx.accounts.initiator_ata.key(),
            &ctx.accounts.bet.key(),
            &[],
            amount
        )?;

        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.bet.to_account_info(),
                ctx.accounts.admin_ata.to_account_info(),
                ctx.accounts.initiator_ata.to_account_info(),
            ],
            &[&signer_seeds[..]],
        )?;

        let signers = &[
            SOL_VAULT.as_bytes(),
            &[vault_bump],
        ];

        let ix = solana_program::system_instruction::transfer(
            ctx.accounts.sol_vault.key, 
            ctx.accounts.initiator.key, 
            amount
        );
    
        invoke_signed(
            &ix, 
            &[
                ctx.accounts.sol_vault, 
                ctx.accounts.initiator, 
                ctx.accounts.system_program
            ], 
            signers
        );
        // anchor_spl::token::transfer(
        //     CpiContext::new_with_signer(
        //         ctx.accounts.token_program.to_account_info(),
        //         cpi_accounts,
        //         &[&signer_seeds[..]],
        //     ),
        //     calculate_probability(
        //         &ctx.accounts.bet,
        //         ctx.accounts.vault.coin_info.amount,
        //         &requester.random,
        //     ) as u64,
        // )?;

        remove_pda(
            &ctx.accounts.initiator.to_account_info(),
            &ctx.accounts.vault.to_account_info(),
        )?;

        return Ok(());
    }

    pub fn admin_bet(
        ctx: Context<AdminBet>,
        fee_finality: u64,
        fee_epoch: u64,
        fee_cluster: u64,
        fee_lamport: u64,
        amount_finality: u64,
        amount_epoch: u64,
        amount_cluster: u64,
        amount_lamport: u64,
    ) -> Result<()> {
        let bet = &mut ctx.accounts.bet;

        if ctx.accounts.authority.key() != admin_account_pubkey()? {
            return Err(DegenErrorCode::Unauthorized.into());
        }

        bet.fee_cluster = fee_cluster;
        bet.fee_epoch = fee_epoch;
        bet.fee_finality = fee_finality;
        bet.fee_lamport = fee_lamport;

        bet.amount_cluster = amount_cluster;
        bet.amount_epoch = amount_epoch;
        bet.amount_finality = amount_finality;
        bet.amount_lamport = amount_lamport;

        anchor_spl::token::approve(ctx.accounts.token_approve_ctx(), ctx.accounts.mint.supply)?;

        Ok(())
    }

    pub fn update_admin_fee(
        ctx: Context<UpdateAdminFee>,
        fee_finality: u64,
        fee_epoch: u64,
        fee_cluster: u64,
        fee_lamport: u64,
    ) -> Result<()> {
        let bet = &mut ctx.accounts.bet;

        if ctx.accounts.authority.key() != admin_account_pubkey()? {
            return Err(DegenErrorCode::Unauthorized.into());
        }

        bet.fee_cluster = fee_cluster;
        bet.fee_epoch = fee_epoch;
        bet.fee_finality = fee_finality;
        bet.fee_lamport = fee_lamport;

        Ok(())
    }

    pub fn update_bet_amount(
        ctx: Context<UpdateBetAmount>,
        amount_finality: u64,
        amount_epoch: u64,
        amount_cluster: u64,
        amount_lamport: u64,
    ) -> Result<()> {
        let bet = &mut ctx.accounts.bet;

        if ctx.accounts.authority.key() != admin_account_pubkey()? {
            return Err(DegenErrorCode::Unauthorized.into());
        }

        bet.amount_cluster = amount_cluster;
        bet.amount_epoch = amount_epoch;
        bet.amount_finality = amount_finality;
        bet.amount_lamport = amount_lamport;

        Ok(())
    }

    pub fn update_admin_ata(ctx: Context<UpdateAdminAta>) -> Result<()> {
        if ctx.accounts.authority.key() != admin_account_pubkey()? {
            return Err(DegenErrorCode::Unauthorized.into());
        }

        anchor_spl::token::approve(ctx.accounts.token_approve_ctx(), ctx.accounts.mint.supply)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateCoin<'info> {
    #[account(
        init,
        seeds = [VAULT_PREFIX.as_bytes(), mint.key().as_ref(), initiator.key().as_ref(), crate::id().as_ref(),],
        bump,
        payer = initiator,
        space = 8 + size_of::<Vault>()
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [SOL_VAULT.as_bytes()],
        bump,
    )]
    pub sol_vault: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: fee
    pub bet: Account<'info, Bet>,

    /// CHECK: PDA for calling the Oracle for random number
    #[account(mut)]
    pub requester: AccountInfo<'info>,

    /// CHECK: Initiator of the flip
    #[account(mut)]
    pub initiator: Signer<'info>,

    /// CHECK: Initiator Token ATA
    #[account(mut)]
    pub initiator_ata: Account<'info, TokenAccount>,

    /// CHECK: Account making the random request
    #[account(mut)]
    pub oracle: AccountInfo<'info>,

    /// CHECK: Token A mint
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA holding the coin toss info.
    #[account(mut)]
    pub oracle_vault: AccountInfo<'info>,

    /// CHECK: Admin account to receive fee.
    #[account(mut)]
    pub admin: AccountInfo<'info>,

    /// CHECK: Admin Token A ATA to receive tokens.
    #[account(mut)]
    pub admin_ata: Box<Account<'info, TokenAccount>>,

    /// CHECK: The program responsible for generating randomness and holding the random number.
    pub degenrand_program: AccountInfo<'info>,

    /// CHECK: System Variable for getting rent to create a PDA.
    pub rent: Sysvar<'info, Rent>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateCoin<'info> {
    pub fn token_transfer_ctx(
        &self,
        authority: AccountInfo<'info>,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            authority: authority.to_account_info(),
            from,
            to,
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn request_random(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, degenrand::cpi::accounts::RequestRandom<'info>> {
        let cpi_accounts = degenrand::cpi::accounts::RequestRandom {
            requester: self.requester.to_account_info(),
            vault: self.oracle_vault.clone(),
            authority: self.initiator.to_account_info(),
            oracle: self.oracle.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        CpiContext::new(self.degenrand_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct RevealCoin<'info> {
    /// CHECK: The account flipping
    #[account(mut, signer)]
    pub initiator: AccountInfo<'info>,

    /// CHECK: Initiator Token B ATA
    #[account(mut)]
    pub initiator_ata: Box<Account<'info, TokenAccount>>,

    /// CHECK: Admin Token B ATA
    #[account(mut)]
    pub admin_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [SOL_VAULT.as_bytes()],
        bump,
    )]
    pub sol_vault: AccountInfo<'info>,

    /// CHECK: Token A mint
    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: PDA storing which is the authority for both ATAs
    #[account(mut)]
    pub vault: Box<Account<'info, Vault>>,

    /// CHECK: PDA for calling the Oracle for random number
    #[account(mut)]
    pub requester: AccountInfo<'info>,

    /// CHECK: bet info
    #[account(mut)]
    pub bet: Account<'info, Bet>,

    /// CHECK: degenrand program
    pub degenrand_program: AccountInfo<'info>,

    // pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminBet<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [ADMIN_BET_PREFIX.as_bytes(), admin_account_pubkey()?.as_ref()],
        bump,
        space = 8 + size_of::<Bet>(),
    )]
    /// CHECK: fee
    pub bet: Account<'info, Bet>,

    /// CHECK: initiator to return amount to
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Token B mint
    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: Admin Token B ATA
    #[account(mut)]
    pub admin_ata: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

impl<'info> AdminBet<'info> {
    pub fn token_approve_ctx<'b, 'c>(&self) -> CpiContext<'_, 'b, 'c, 'info, Approve<'info>> {
        let cpi_accounts = Approve {
            delegate: self.bet.to_account_info(),
            authority: self.authority.to_account_info(),
            to: self.admin_ata.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct UpdateAdminFee<'info> {
    /// CHECK: initiator to return amount to
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: bet info
    #[account(mut)]
    pub bet: Account<'info, Bet>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateBetAmount<'info> {
    /// CHECK: initiator to return amount to
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: bet info
    #[account(mut)]
    pub bet: Account<'info, Bet>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAdminAta<'info> {
    /// CHECK: initiator to return amount to
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: bet info
    #[account(mut)]
    pub bet: Account<'info, Bet>,

    /// CHECK: Admin Token B ATA
    #[account(mut)]
    pub admin_ata: Box<Account<'info, TokenAccount>>,

    /// CHECK: Token B mint
    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

impl<'info> UpdateAdminAta<'info> {
    pub fn token_approve_ctx<'b, 'c>(&self) -> CpiContext<'_, 'b, 'c, 'info, Approve<'info>> {
        let cpi_accounts = Approve {
            delegate: self.bet.to_account_info(),
            authority: self.authority.to_account_info(),
            to: self.admin_ata.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Debug, Default, AnchorDeserialize, AnchorSerialize, Clone)]
pub struct CoinInfo {
    initiator: Pubkey,
    amount: u64,
    sol_amount: u64,
    multiplier: u64,
}

#[account]
#[derive(Debug, Default)]
pub struct Bet {
    fee_finality: u64,
    fee_epoch: u64,
    fee_cluster: u64,
    fee_lamport: u64,

    amount_finality: u64,
    amount_epoch: u64,
    amount_cluster: u64,
    amount_lamport: u64,
}

// Used for holding the sol balance and transfering to winner
#[account]
#[derive(Debug, Default)]
pub struct Vault {
    pub coin_info: CoinInfo,
}

#[error_code]
pub enum DegenErrorCode {
    #[msg("You are not authorized to complete this transaction")]
    Unauthorized,

    #[msg("The coin is has already been flipped")]
    AlreadyCompleted,

    #[msg("A coin is already flipping. Only one flip may be made at a time")]
    InflightRequest,

    #[msg("The Oracle has not provided a response yet")]
    OracleNotCompleted,

    #[msg("Admin Token Pubkey Invalid")]
    InvalidAdminPubkey,

    #[msg("Not enough Tokens")]
    NotEnoughTokens,

    #[msg("Failed to understand Instruction")]
    FallBacked,

    #[msg("Account Mismatch")]
    AccountMismatch,
}
