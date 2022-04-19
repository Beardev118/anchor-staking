import { Keypair, PublicKey } from "@solana/web3.js";
import { createMint, getMint } from "@solana/spl-token";
import {
    beefMintKeypair,
    stakeMintKeypair,
    connection,
    randomPayer,
    findStakeMintAuthorityPDA,
} from "./config";
import * as anchor from "@project-serum/anchor";

const program = anchor.workspace.AnchorStaking;
const provider = anchor.Provider.env();

const createMints = async () => {

    let beefMintAddress;
    let stakeMintAddress;
    try {
        beefMintAddress = await getMint(provider.connection, beefMintKeypair.publicKey);
    } catch (error) {
        beefMintAddress = await createMintAcct(
            beefMintKeypair,
            beefMintKeypair.publicKey
        );
    }

    const [stakePDA, _] =  await findStakeMintAuthorityPDA();
    // const [stakePDA, stakePDABump] = await PublicKey.findProgramAddress(
    //     [stakeMintAddress.toBuffer()],
    //     program.programId
    // );

    try {
        stakeMintAddress = await getMint(provider.connection, stakeMintKeypair.publicKey);
    } catch (error) {
        stakeMintAddress = await createMintAcct(
            stakeMintKeypair,
            stakePDA
        );
    }


    console.log(`🐮 beef Mint Address: ${beefMintAddress}`);
    console.log(`🥩️ stake Mint Address: ${stakeMintAddress}`);
}



const createMintAcct = async (keypairToAssign: Keypair, authorityToAssign: PublicKey): Promise<PublicKey> => {
    return await createMint(
        connection,
        await randomPayer(),
        authorityToAssign, // mint authority
        null, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
        8, // decimals
        keypairToAssign // address of the mint
    );
}


export {
    createMints
}