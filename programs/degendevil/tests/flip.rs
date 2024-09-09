#![cfg(feature = "test-bpf")]

mod utils;

use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::{
    account::AccountSharedData, instruction::AccountMeta, program_pack::Pack,
    transaction::Transaction,
};
use {
    solana_program_test::*,
    solana_sdk::{instruction::Instruction, signature::Signer},
    utils::*,
};

#[tokio::test]
async fn flip() -> Result<(), Error> {
    let TestContext {
        mut ctx,
        alice,
        oracle,
        ..
    } = get_program_test_context().await?;

    let admin_keypair = keypair_from_file("./tests/fixtures/id.json")?;

    ctx.set_account(
        &admin_keypair.pubkey(),
        &AccountSharedData::new(
            100000000000000000,
            0,
            &anchor_lang::solana_program::system_program::id(),
        )
        .into(),
    );

    let token_x = create_token(2, &mut ctx).await?;
    let token_y = create_token(0, &mut ctx).await?;

    let alice_token_x_account = create_token_account(&token_x.pubkey(), &alice, &mut ctx).await?;
    let alice_token_y_account = create_token_account(&token_y.pubkey(), &alice, &mut ctx).await?;

    let admin_x_ata = create_token_account(&token_x.pubkey(), &admin_keypair, &mut ctx).await?;
    let admin_y_ata = create_token_account(&token_y.pubkey(), &admin_keypair, &mut ctx).await?;

    mint_token(
        &token_x.pubkey(),
        &alice_token_x_account.pubkey(),
        1000000,
        &mut ctx,
    )
    .await?;

    mint_token(&token_y.pubkey(), &admin_y_ata.pubkey(), 52500000, &mut ctx).await?;

    let amount = 5250;

    let (vault_pda, _) = degendevil::vault_pda(&token_x.pubkey(), &alice.pubkey());

    let (requester, req_bump) = degenrand::requestor_pda(&alice.pubkey());
    let (oracle_vault, oracle_bump) = degenrand::vault_pda(&alice.pubkey());

    let (admin_bet_pda, _) = degendevil::admin_bet_pda()?;

    println!(
        "\n\nAdmin Balance {:#?}",
        ctx.banks_client.get_balance(admin_keypair.pubkey()).await?
    );

    if let Some(account) = ctx.banks_client.get_account(admin_y_ata.pubkey()).await? {
        let token_account = spl_token::state::Account::unpack(account.data.as_slice())?;
        println!("{:#?}", token_account);
    }

    let degendevil_admin_fee_accounts = degendevil::accounts::AdminBet {
        authority: admin_keypair.pubkey(),
        bet: admin_bet_pda,
        mint: token_x.pubkey(),
        admin_ata: admin_y_ata.pubkey(),
        token_program: spl_token::id(),
        system_program: anchor_lang::solana_program::system_program::id(),
    }
    .to_account_metas(None);

    let degendevil_admin_fee_data = degendevil::instruction::AdminBet {
        fee_finality: 12_000_000,
        fee_epoch: 20_000_000,
        fee_cluster: 25_000_000,
        fee_lamport: 30_000_000,
        amount_finality: 700,
        amount_epoch: 1750,
        amount_cluster: 3500,
        amount_lamport: 5250,
    }
    .data();

    let transaction = Transaction::new_signed_with_payer(
        &[Instruction {
            accounts: degendevil_admin_fee_accounts,
            data: degendevil_admin_fee_data,
            program_id: degendevil::id(),
        }],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &admin_keypair],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(transaction).await?;

    let degenrand_init_accounts = degenrand::accounts::Initialize {
        authority: alice.pubkey(),
        oracle: oracle.pubkey(),
        requester,
        vault: oracle_vault,
        rent: anchor_lang::solana_program::sysvar::rent::id(),
        system_program: anchor_lang::solana_program::system_program::id(),
    }
    .to_account_metas(None);

    let degenrand_init_data = degenrand::instruction::Initialize {
        request_bump: req_bump,
        vault_bump: oracle_bump,
    }
    .data();

    let transaction = Transaction::new_signed_with_payer(
        &[Instruction {
            accounts: degenrand_init_accounts,
            data: degenrand_init_data,
            program_id: degenrand::id(),
        }],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &alice],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(transaction).await?;

    println!(
        "\n\nAlice Balance {:#?}",
        ctx.banks_client.get_balance(alice.pubkey()).await?
    );

    let create_coin_accounts = degendevil::accounts::CreateCoin {
        vault: vault_pda,
        initiator: alice.pubkey(),
        requester,
        admin: admin_keypair.pubkey(),
        bet: admin_bet_pda,
        admin_ata: admin_x_ata.pubkey(),
        initiator_ata: alice_token_x_account.pubkey(),
        mint: token_x.pubkey(),
        oracle: oracle.pubkey(),
        oracle_vault,
        degenrand_program: degenrand::id(),
        rent: anchor_lang::solana_program::sysvar::rent::id(),
        token_program: spl_token::id(),
        system_program: anchor_lang::solana_program::system_program::id(),
    }
    .to_account_metas(None);

    let create_coin_data = degendevil::instruction::CreateCoin { amount }.data();

    let ix = Instruction {
        program_id: degendevil::id(),
        accounts: create_coin_accounts,
        data: create_coin_data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &alice],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(transaction).await?;

    println!(
        "\n\nAlice Balance {:#?}",
        ctx.banks_client.get_balance(alice.pubkey()).await?
    );

    let mut publish_random_accounts = degenrand::accounts::PublishRandom {
        oracle: oracle.pubkey(),

        system_program: anchor_lang::solana_program::system_program::id(),
    }
    .to_account_metas(None);

    publish_random_accounts.push(AccountMeta::new(requester, false));

    // Force to win.
    let publish_random_data = degenrand::instruction::PublishRandom {
        pkt_id: [1u8; 32],
        random: [1u8; 64],
        tls_id: [1u8; 32],
    }
    .data();

    let ix = Instruction {
        program_id: degenrand::id(),
        accounts: publish_random_accounts,
        data: publish_random_data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &oracle],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(transaction).await?;

    let reveal_coin_data = degendevil::instruction::RevealCoin {}.data();

    let reveal_coin_accounts = degendevil::accounts::RevealCoin {
        initiator: alice.pubkey(),
        initiator_ata: alice_token_y_account.pubkey(),
        admin_ata: admin_y_ata.pubkey(),
        mint: token_x.pubkey(),
        vault: vault_pda,
        bet: admin_bet_pda,
        requester,
        degenrand_program: degenrand::id(),
        token_program: spl_token::id(),
        system_program: anchor_lang::solana_program::system_program::id(),
    }
    .to_account_metas(None);

    let ix = Instruction {
        program_id: degendevil::id(),
        accounts: reveal_coin_accounts,
        data: reveal_coin_data,
    };
    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &alice],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(transaction).await?;

    println!(
        "\n\nAlice Balance {:#?}",
        ctx.banks_client.get_balance(alice.pubkey()).await?
    );

    println!(
        "\n\nAdmin Balance {:#?}",
        ctx.banks_client.get_balance(admin_keypair.pubkey()).await?
    );

    if let Some(account) = ctx.banks_client.get_account(admin_y_ata.pubkey()).await? {
        let token_account = spl_token::state::Account::unpack(account.data.as_slice())?;
        println!("{:#?}", token_account);
    }

    Ok(())
}
