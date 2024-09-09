import * as anchor from '@project-serum/anchor';
import { Program, } from '@project-serum/anchor';
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Degendevil } from '../target/types/degendevil';
import { feePDA, getKeypair, getPublicKey } from '../tests/utils';

describe('degendevil', () => {


    // const ENV = 'http://localhost:8899';
    const ENV = "https://api.devnet.solana.com";
    // const ENV = "https://api.mainnet-beta.solana.com";


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


    const mintAuth = getKeypair("id");
    let mintAuthProvider = createProvider(mintAuth);
    const mintAuthProgram = new anchor.Program(program.idl, program.programId, mintAuthProvider);
    const adminAta = getPublicKey("admin_mint_x");
    const mintYPublicKey = getPublicKey("mint_y");
    const adminYAta = getPublicKey("admin_mint_y");
    let feePdaAddress, feeBump;



    it('Update Admin Fee', async () => {
        let fp = await feePDA(mintAuth.publicKey, program.programId);
        feePdaAddress = fp.feePdaAddress, feeBump = fp.feeBump;

        console.log('Fee account: ', feePdaAddress.toString());
        console.log("Admin Pubkey", mintAuth.publicKey.toString());

        let mintY = new Token(mintAuthProvider.connection, mintYPublicKey, TOKEN_PROGRAM_ID, mintAuth);
        let mintInfo = await mintY.getMintInfo();
        console.log(
            mintInfo.supply

        );

        try {

            anchor.setProvider(mintAuthProvider);
            await mintAuthProgram.rpc.updateAdminAta(
                {
                    accounts: {
                        authority: mintAuth.publicKey,
                        adminAta: adminYAta,
                        bet: feePdaAddress,
                        mint: mintY.publicKey,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        systemProgram: anchor.web3.SystemProgram.programId,
                    },
                    signers: [mintAuth]
                }
            );
        } catch (e) {
            console.log("Couldn't Update Fee");

        }
    });

});