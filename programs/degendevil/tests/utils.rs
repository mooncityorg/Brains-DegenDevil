use degendevil::id;

use {
    anchor_lang::solana_program::system_instruction::create_account,
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        native_token::*,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
    },
    spl_token::{
        self,
        instruction::*,
        state::{Account, Mint},
    },
};

const PROGRAM_NAME: &'static str = "degendevil";

pub type Error = Box<dyn std::error::Error>;

pub type CommandResult = Result<(), Error>;

pub async fn create_token(decimals: u8, ctx: &mut ProgramTestContext) -> Result<Keypair, Error> {
    let token = Keypair::new();

    let rent = ctx.banks_client.get_rent().await?;
    let minimum_balance_for_rent_exemption = rent.minimum_balance(Mint::LEN);

    let transaction = Transaction::new_signed_with_payer(
        &[
            solana_sdk::system_instruction::create_account(
                &ctx.payer.pubkey(),
                &token.pubkey(),
                minimum_balance_for_rent_exemption,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            initialize_mint(
                &spl_token::id(),
                &token.pubkey(),
                &ctx.payer.pubkey(),
                None,
                decimals,
            )?,
        ],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &token],
        ctx.last_blockhash,
    );

    check_fee_payer_balance(ctx, minimum_balance_for_rent_exemption).await?;

    ctx.banks_client.process_transaction(transaction).await?;
    Ok(token)
}

pub async fn create_token_account(
    token: &Pubkey,
    owner: &Keypair,
    ctx: &mut ProgramTestContext,
) -> Result<Keypair, Error> {
    let account = Keypair::new();

    let rent = ctx.banks_client.get_rent().await?;
    let minimum_balance_for_rent_exemption = rent.minimum_balance(Account::LEN);

    let transaction = Transaction::new_signed_with_payer(
        &[
            create_account(
                &ctx.payer.pubkey(),
                &account.pubkey(),
                minimum_balance_for_rent_exemption,
                Account::LEN as u64,
                &spl_token::id(),
            ),
            initialize_account(&spl_token::id(), &account.pubkey(), &token, &owner.pubkey())?,
        ],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &account],
        ctx.last_blockhash,
    );

    check_fee_payer_balance(ctx, minimum_balance_for_rent_exemption).await?;

    ctx.banks_client.process_transaction(transaction).await?;
    Ok(account)
}

pub async fn mint_token(
    token: &Pubkey,
    recipient: &Pubkey,
    amount: u64,
    ctx: &mut ProgramTestContext,
) -> CommandResult {
    let transaction = Transaction::new_signed_with_payer(
        &[mint_to(
            &spl_token::id(),
            &token,
            &recipient,
            &ctx.payer.pubkey(),
            &[&ctx.payer.pubkey()],
            amount,
        )?],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );

    check_fee_payer_balance(ctx, 0).await?;

    Ok(ctx.banks_client.process_transaction(transaction).await?)
}

async fn check_fee_payer_balance(
    ctx: &mut ProgramTestContext,
    required_balance: u64,
) -> Result<(), Error> {
    let fee_payer = ctx.payer.pubkey();
    let balance = ctx.banks_client.get_balance(fee_payer).await?;

    if balance < required_balance {
        Err(format!(
            "Fee payer, {}, has insufficient balance: {} required, {} available",
            fee_payer,
            lamports_to_sol(required_balance),
            lamports_to_sol(balance)
        )
        .into())
    } else {
        Ok(())
    }
}
pub struct TestContext {
    pub ctx: ProgramTestContext,
    pub alice: Keypair,
    pub bob: Keypair,
    pub oracle: Keypair,
    pub winner_mint_holder: Keypair,
}

pub async fn get_program_test_context() -> Result<TestContext, Error> {
    let program_id = id();

    let mut test_validator =
        ProgramTest::new(PROGRAM_NAME, program_id, processor!(degendevil::entry));

    test_validator.add_program("degenrand", degenrand::id(), None);

    let (alice, a_acc) = get_keypair_and_account(10000000000000000);
    let (bob, b_acc) = get_keypair_and_account(10000000000000000);
    let (oracle, oracle_acc) = get_keypair_and_account(10000000000000000);
    let (winner_mint_holder, winner_mint_holder_acc) = get_keypair_and_account(10000000000000000);

    test_validator.add_account(alice.pubkey(), a_acc.into());
    test_validator.add_account(bob.pubkey(), b_acc.into());
    test_validator.add_account(oracle.pubkey(), oracle_acc.into());
    test_validator.add_account(winner_mint_holder.pubkey(), winner_mint_holder_acc.into());

    let ctx = test_validator.start_with_context().await;

    Ok(TestContext {
        ctx,
        alice,
        bob,
        oracle,
        winner_mint_holder,
    })
}

pub fn get_keypair_and_account(amount: u64) -> (Keypair, AccountSharedData) {
    (
        Keypair::new(),
        AccountSharedData::new(
            amount,
            0,
            &anchor_lang::solana_program::system_program::id(),
        ),
    )
}

pub fn keypair_from_file(path: &str) -> Result<Keypair, Error> {
    let mut raw = std::io::Cursor::new(std::fs::read(path)?);
    solana_sdk::signature::read_keypair(&mut raw)
}
