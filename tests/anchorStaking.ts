import { expect } from 'chai';
import * as anchor from "@project-serum/anchor";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  stakeMintAddress,
  beefMintAddress,
  program,
  findStakeMintAuthorityPDA,
  findBeefMintAuthorityPDA
} from "../scripts/config"
import { User } from "./user";
import { createMints } from "../scripts/create-mints";
import { airdropBeef } from "../scripts/airdrop-beef";
import { TokenHelper } from './token_helper';
import { token } from '@project-serum/anchor/dist/cjs/utils';



describe("staker", () => {

  before(async () => {
    await createMints();
    await airdropBeef();
  });

  it("It creates the program $beef token bag",async () => {
    const user = new User();
    const [beefPDA, _] = await getProgramBeefTokenBagPDA();

    await program.rpc.createBeefTokenBag({
      accounts: {
        beefMint: beefMintAddress,
        programBeefTokenBag: beefPDA,
        payer: user.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      }
    });

    const tokenHelper = new TokenHelper(beefMintAddress);
    expect(await tokenHelper.balance(beefPDA)).to.be.eql(0);
  })

  it('Swap $🐮 for $🥩', async () => {

    // 0. Prepare Token Bags
    const user = new User();
    
    await user.getOrCreateStakeTokenBag();
    await user.getOrCreateBeefTokenBag();

    // 1. Get current stake amount
    const userStakes = await user.stakeBalance();
    const userBeefs = await user.beefBalance();

    // For MINT
    // PDA with stakeMint as seed is the stakeMintAuthority
    const [stakePDA, stakePDABump] = await findStakeMintAuthorityPDA();

    // For the TRANSFER
    const [beefBagPDA, beefBagBump] = await findBeefMintAuthorityPDA();

    // 2. Execute our stuff
    await program.rpc.stake(
      stakePDABump,
      beefBagBump,
        new anchor.BN(5_000),
        {
          accounts: {
            // Solana is lost: where are my spl program friends?
            tokenProgram: TOKEN_PROGRAM_ID,

            // *********
            // MINT PART
            // *********

            // Token Program asks: 🏭 what type of token am I supposed to print?
            stakeMint: stakeMintAddress,

            // Token Program asks: ✍️ who is allowed to print tokens from stakeMint?
            stakeMintAuthority: stakePDA,

            // 💰🥩 Token Program wonders: "where should I mint this to?"
            userStakeTokenBag: user.stakeTokenBag,

            // *******************
            // TRANSFER PART
            // *******************

            userBeefTokenBag: user.beefTokenBag,
            userBeefTokenBagAuthority: user.wallet.publicKey,
            programBeefTokenBag: beefBagPDA,
            beefMint: beefMintAddress
          },
        },
    );

    // 3. Test
    // We expect the user to have received 5_000 stakes $🥩
    expect(await user.stakeBalance()).to.be.eql(userStakes + 5_000);

    // We expect the user ot have paid 5_000 $beef to the program.
    expect(await user.beefBalance()).to.be.eql(userBeefs - 5_000);
    const tokenHelper = new TokenHelper(beefMintAddress);
    expect(await tokenHelper.balance(beefBagPDA)).to.be.eql(5_000);
  });

  it("It redeems $stake for $beef", async () => {
    // 0. Prepare token bag
    const user = new User();
    await user.getOrCreateStakeTokenBag();
    await user.getOrCreateBeefTokenBag();
    const userStakes = await user.stakeBalance();
    const userBeefs = await user.beefBalance();

    // For the TRANSFER
    const [beefBagPDA, beefBagBump] = await getProgramBeefTokenBagPDA();

    await program.rpc.unstake(beefBagBump, new anchor.BN(5_000),
      {
        accounts: {
          tokenProgram: TOKEN_PROGRAM_ID,

          stakeMint: stakeMintAddress,
          userStakeTokenBag: user.stakeTokenBag,
          userStakeTokenBagAuthority: user.wallet.publicKey,

          programBeefTokenBag: beefBagPDA,
          userBeefTokenBag: user.beefTokenBag,
          beefMint: beefMintAddress,
        },
      }
    );

    // 3. Tests
    // We expect the user to have redeem $stake to the program
    expect(await user.stakeBalance()).to.be.eql(userStakes - 5_000);

    // We expect the user to have received 5_000 beef $beef back
    expect(await user.beefBalance()).to.be.eql(userBeefs + 5_000);
  });
});


const getProgramBeefTokenBagPDA = async (): Promise<[PublicKey, number]> => {
  const seed = beefMintAddress;

  return await PublicKey.findProgramAddress(
      [seed.toBuffer()],
      program.programId
  );
}