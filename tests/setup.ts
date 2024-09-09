import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { Connection, Keypair, PublicKey, Signer, } from "@solana/web3.js";
import { getKeypair, getTokenBalance, writePublicKey } from "./utils";

const mintToken = (
    connection: Connection,
    { publicKey, secretKey }: Signer,

    decimal: number
) => {

    return Token.createMint(
        connection,
        {
            publicKey,
            secretKey,
        },
        publicKey,
        null,
        decimal,
        TOKEN_PROGRAM_ID
    );
};

const setupMint = async (
    connection: Connection,
    aliceKeypair: Keypair,
    bobKeypair: Keypair,
    mintAuthority: Signer
): Promise<[PublicKey, PublicKey, PublicKey, PublicKey, PublicKey, PublicKey, PublicKey, PublicKey]> => {

    const mintX = await mintToken(connection, mintAuthority, 2);
    writePublicKey(mintX.publicKey, `mint_${"X".toLowerCase()}`);
    console.log(`Creating Mint X...${mintX.publicKey.toString()}`);

    const mintY = await mintToken(connection, mintAuthority, 0);
    writePublicKey(mintY.publicKey, `mint_${"Y".toLowerCase()}`);
    console.log(`Creating Mint Y...${mintY.publicKey.toString()}`);

    console.log(`Creating Alice TokenAccount for X...`);
    const aliceXTokenAccount = await mintX.createAssociatedTokenAccount(aliceKeypair.publicKey);
    writePublicKey(aliceXTokenAccount, `alice_${"X".toLowerCase()}`);

    console.log(`Creating Alice TokenAccount for Y...`);
    const aliceYTokenAccount = await mintY.createAssociatedTokenAccount(aliceKeypair.publicKey);
    writePublicKey(aliceYTokenAccount, `alice_${"Y".toLowerCase()}`);

    console.log(`Creating Bob TokenAccount for X...`);
    const bobXTokenAccount = await mintX.createAssociatedTokenAccount(bobKeypair.publicKey);
    writePublicKey(bobXTokenAccount, `bob_${"X".toLowerCase()}`);

    console.log(`Creating Bob TokenAccount for Y...`);
    const bobYTokenAccount = await mintY.createAssociatedTokenAccount(bobKeypair.publicKey);
    writePublicKey(bobYTokenAccount, `bob_${"Y".toLowerCase()}`);

    console.log(`Creating Admin Mint TokenAccount for Y...`);
    const adminTokenXAccount = await mintX.createAssociatedTokenAccount(mintAuthority.publicKey);
    writePublicKey(adminTokenXAccount, `admin_mint_x`);

    console.log(`Creating Admin Mint TokenAccount for Y...`);
    const adminTokenYAccount = await mintY.createAssociatedTokenAccount(mintAuthority.publicKey);
    writePublicKey(adminTokenYAccount, `admin_mint_y`);
    try {

        await mintX.mintTo(aliceXTokenAccount, mintAuthority, [], 1000_00);
        await mintX.mintTo(bobXTokenAccount, mintAuthority, [], 1000_00);
        await mintX.mintTo(adminTokenXAccount, mintAuthority, [], 1_000_000_000_00);

        await mintY.mintTo(adminTokenYAccount, mintAuthority, [], 1_000_000_000);
        console.log("Minting Tokens done");

    } catch (e) {
        console.log("Error : Minting ", e);

    }



    return [mintX.publicKey, mintY.publicKey, aliceXTokenAccount, bobXTokenAccount, aliceYTokenAccount, bobYTokenAccount, adminTokenXAccount, adminTokenYAccount];
};


export const setup = async (cluster: string) => {
    const aliceKeypair = getKeypair("alice");
    const bobKeypair = getKeypair("bob");
    const mintAuthority = getKeypair("id");

    const connection = new Connection(cluster, "confirmed");


    const [mintX, mintY, aliceXTokenAccount, bobXTokenAccount, aliceYTokenAccount, bobYTokenAccount, adminTokenXAccount, adminTokenYAccount] = await setupMint(
        connection,
        aliceKeypair,
        bobKeypair,
        mintAuthority,
    );

    console.log("✨Setup complete✨\n");
    console.table([
        {
            "Alice Token Account X": await getTokenBalance(
                aliceXTokenAccount,
                connection
            ),

            "Bob Token Account X": await getTokenBalance(
                bobXTokenAccount,
                connection
            ),
            "Alice Token Account Y": await getTokenBalance(
                aliceYTokenAccount,
                connection
            ),

            "Bob Token Account Y": await getTokenBalance(

                bobYTokenAccount,
                connection
            ),

            "Admin Mint Token Account X": await getTokenBalance(
                adminTokenXAccount,
                connection
            ),

            "Admin Mint Token Account Y": await getTokenBalance(
                adminTokenYAccount,
                connection
            ),

        },
    ]);
    console.log("");
};

