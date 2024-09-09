import * as anchor from '@project-serum/anchor';
import { Program, } from '@project-serum/anchor';
import { Degendevil } from '../target/types/degendevil';
import { feePDA, getKeypair } from '../tests/utils';

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

    let feePdaAddress, feeBump;

    // Bet types and Fee
    // Change Values here and run "anchor run update"
    let feeFinality = new anchor.BN(12_000_000);
    let feeEpoch = new anchor.BN(20_000_000);
    let feeCluster = new anchor.BN(25_000_000);
    let feeLamport = new anchor.BN(30_000_000);

    it('Update Admin Fee', async () => {
        let fp = await feePDA(mintAuth.publicKey, program.programId);
        feePdaAddress = fp.feePdaAddress, feeBump = fp.feeBump;

        console.log('Fee account: ', feePdaAddress.toString());
        console.log("Admin Pubkey", mintAuth.publicKey.toString());

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

});