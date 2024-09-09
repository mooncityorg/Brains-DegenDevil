import * as anchor from '@project-serum/anchor';
import { Program, } from '@project-serum/anchor';
import { Degendevil } from '../target/types/degendevil';
import { randomBytes } from 'crypto';
import { MockOracleSession as OracleSession } from "./sessions.js";
import { feePDA, getKeypair, getPublicKey, oracleVaultPda, requestorPda, vaultPda, } from './utils';
import { setup } from './setup';
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';


describe('degendevil', () => {

    // const ENV = 'http://localhost:8899';
    const ENV = "https://api.devnet.solana.com";
    const degenrandId = new anchor.web3.PublicKey(anchor.workspace.Degenrand.programId);


    function createProvider(keyPair) {
        let solConnection = new anchor.web3.Connection(ENV);
        let walletWrapper = new anchor.Wallet(keyPair);
        return new anchor.Provider(solConnection, walletWrapper, {
            preflightCommitment: 'recent',
        });
    }

    async function getBalance(prov, key) {
        anchor.setProvider(prov);
        return await prov.connection.getBalance(key, "confirmed");
    }

    const program = anchor.workspace.Degendevil as Program<Degendevil>;

    const userKeyPair = getKeypair("alice");
    let provider1 = createProvider(userKeyPair);
    const user1Program = new anchor.Program(program.idl, program.programId, provider1);

    const mintAuth = getKeypair("id");
    let mintAuthProvider = createProvider(mintAuth);
    const mintAuthProgram = new anchor.Program(program.idl, program.programId, mintAuthProvider);

    const oracle = getKeypair("oracle");
    const oracleSession = new OracleSession(oracle, anchor.workspace.Degenrand.idl, degenrandId, ENV);

    const oraclePubkey = oracle.publicKey;
    const degenrandProgram = new anchor.Program(anchor.workspace.Degenrand.idl, degenrandId, provider1);

    const amount = new anchor.BN(5250);


    let mintX;
    let mintY;
    let initiatorAta;
    let initiatorYAta;
    let adminAta;
    let adminYAta;

    let requestorPdaAddress, reqBump;
    let oracleVaultPdaAddress, reqVaultBump;
    let feePdaAddress, feeBump;
    let vaultPdaAddress, vaultBump;

    // Bet types and Fee
    let feeFinality = new anchor.BN(12_000_000);
    let feeEpoch = new anchor.BN(20_000_000);
    let feeCluster = new anchor.BN(25_000_000);
    let feeLamport = new anchor.BN(30_000_000);

    let amountFinality = new anchor.BN(700);
    let amountEpoch = new anchor.BN(1750);
    let amountCluster = new anchor.BN(3500);
    let amountLamport = new anchor.BN(5250);



    anchor.setProvider(provider1);

    it('Set up tests', async () => {

        // await setup(ENV);

        mintX = getPublicKey("mint_x");
        mintY = getPublicKey("mint_y");
        initiatorAta = getPublicKey("alice_x");
        initiatorYAta = getPublicKey("alice_y");
        adminAta = getPublicKey("admin_mint_x");
        adminYAta = getPublicKey("admin_mint_y");

        let rp = await requestorPda(userKeyPair.publicKey, degenrandProgram.programId);
        requestorPdaAddress = rp.requestorPdaAddress; reqBump = rp.reqBump;

        let op = await oracleVaultPda(userKeyPair.publicKey, degenrandProgram.programId);
        oracleVaultPdaAddress = op.oracleVaultPdaAddress; reqVaultBump = op.reqVaultBump;

        let fp = await feePDA(mintAuth.publicKey, program.programId);
        feePdaAddress = fp.feePdaAddress, feeBump = fp.feeBump;

        let vp = await vaultPda(mintX, userKeyPair.publicKey, program.programId);

        vaultPdaAddress = vp.vaultPdaAddress; vaultBump = vp.vaultBump;



        console.log('Req account: ', requestorPdaAddress.toString());
        console.log('Vault account: ', vaultPdaAddress.toString());
        console.log('Req Vault account: ', oracleVaultPdaAddress.toString());
        console.log('Fee account: ', feePdaAddress.toString());
        console.log("Admin Pubkey", mintAuth.publicKey.toString());
    });

    // Uncomment Run once
    // it('Setup Admin Fee!', async () => {
    //     try {

    //         anchor.setProvider(mintAuthProvider);
    //         await mintAuthProgram.rpc.adminBet(
    //             feeFinality,
    //             feeEpoch,
    //             feeCluster,
    //             feeLamport,
    //             amountFinality,
    //             amountEpoch,
    //             amountCluster,
    //             amountLamport,

    //             {
    //                 accounts: {
    //                     authority: mintAuth.publicKey,
    //                     adminAta: adminYAta,
    //                     bet: feePdaAddress,
    // mint: mintY,
    //                     tokenProgram: TOKEN_PROGRAM_ID,
    //                     systemProgram: anchor.web3.SystemProgram.programId,
    //                 },
    //                 signers: [mintAuth]
    //             }
    //         );
    //     } catch (e) {
    //         console.log("Admin Fee Already Setup");

    //     }
    // });

    it('Approve Admin Ata!', async () => {
        try {

            anchor.setProvider(mintAuthProvider);
            await mintAuthProgram.rpc.updateAdminAta(
                {
                    accounts: {
                        authority: mintAuth.publicKey,
                        adminAta: adminYAta,
                        bet: feePdaAddress,
                        mint: mintY,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        systemProgram: anchor.web3.SystemProgram.programId,
                    },
                    signers: [mintAuth]
                }
            );
        } catch (e) {
            console.log("Admin Fee Already Setup");

        }
    });


    it('Update Admin Fee!', async () => {
        try {

            anchor.setProvider(mintAuthProvider);
            await mintAuthProgram.rpc.updateAdminFee(
                feeFinality,
                feeEpoch,
                feeCluster,
                feeLamport,
                {
                    accounts: {
                        authority: mintAuth.publicKey,
                        bet: feePdaAddress,
                        systemProgram: anchor.web3.SystemProgram.programId,
                    },
                    signers: [mintAuth]
                }
            );
        } catch (e) {
            console.log("Couldn't Update Fee");

        }
    });

    it('Update Bet Amounts!', async () => {
        try {

            anchor.setProvider(mintAuthProvider);
            await mintAuthProgram.rpc.updateBetAmount(
                amountFinality,
                amountEpoch,
                amountCluster,
                amountLamport,
                {
                    accounts: {
                        authority: mintAuth.publicKey,
                        bet: feePdaAddress,
                        systemProgram: anchor.web3.SystemProgram.programId,
                    },
                    signers: [mintAuth]
                }
            );
        } catch (e) {
            console.log("Couldn't Update Fee");

        }
    });
    it('Initialize Oracle!', async () => {

        try {

            anchor.setProvider(provider1);
            await degenrandProgram.rpc.initialize(
                reqBump,
                reqVaultBump,
                {
                    accounts: {
                        requester: requestorPdaAddress,
                        vault: oracleVaultPdaAddress,
                        authority: userKeyPair.publicKey,
                        oracle: oraclePubkey,
                        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                        systemProgram: anchor.web3.SystemProgram.programId,
                    },
                    signers: [userKeyPair],
                },
            );
        } catch (e) {
            console.log("Oracle Already Initialized");

        }
    })

    it('Create a coin!', async () => {
        try {

            anchor.setProvider(provider1);
            await user1Program.rpc.createCoin(
                amount,
                {
                    accounts: {
                        vault: vaultPdaAddress,
                        requester: requestorPdaAddress,
                        initiator: userKeyPair.publicKey,
                        initiatorAta,
                        mint: mintX,
                        admin: mintAuth.publicKey,
                        adminAta,
                        bet: feePdaAddress,
                        oracle: oraclePubkey,
                        oracleVault: oracleVaultPdaAddress,
                        degenrandProgram: degenrandId,
                        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        systemProgram: anchor.web3.SystemProgram.programId,
                    },
                    signers: [userKeyPair],
                }
            );
        } catch (e) {
            console.log("Coin already created ", e);

        }
    });


    it('Oracle responds to request', async () => {
        try {

            let randomNumber = randomBytes(64);
            randomNumber[0] = 0;
            randomNumber[1] = 0;
            randomNumber[2] = 0;
            randomNumber[3] = 0;
            randomNumber[4] = 0;
            randomNumber[5] = 0;

            let requester = { publicKey: requestorPdaAddress };

            await oracleSession.publishRandom(requester, randomNumber);
        } catch (e) {
            console.log("Already Published Random", e);

        }
    });

    it('Reveal the result', async () => {
        try {

            anchor.setProvider(provider1);
            await user1Program.rpc.revealCoin(
                {
                    accounts: {
                        initiator: userKeyPair.publicKey,
                        initiatorAta: initiatorYAta,
                        adminAta: adminYAta,
                        vault: vaultPdaAddress,
                        requester: requestorPdaAddress,
                        mint: mintX,
                        bet: feePdaAddress,
                        degenrandProgram: degenrandId,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        systemProgram: anchor.web3.SystemProgram.programId,
                    },

                    signers: [
                        userKeyPair,
                    ],
                },
            );
        } catch (e) {
            console.log("Result Already Revealed", e);

        }
    });


    it('remove pda from degenrand', async () => {
        try {

            anchor.setProvider(provider1);
            await degenrandProgram.rpc.removePdas(
                {
                    accounts: {
                        requester: requestorPdaAddress,
                        vault: oracleVaultPdaAddress,
                        authority: userKeyPair.publicKey,
                        initiator: userKeyPair.publicKey,
                        systemProgram: anchor.web3.SystemProgram.programId,
                    },
                    signers: [userKeyPair],
                },
            );
        } catch (e) {
            console.log("Degenrand PDA already removed.", e);

        }
    });


});
