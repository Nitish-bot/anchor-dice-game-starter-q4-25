import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorDiceGameQ425 } from "../target/types/anchor_dice_game_q4_25";
import { Transaction, Ed25519Program, Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL, SYSVAR_INSTRUCTIONS_PUBKEY, sendAndConfirmTransaction } from "@solana/web3.js";
import { randomBytes } from "crypto"
import { BN } from "bn.js";
import { expect } from "chai";

describe("soldice-anchor", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorDiceGameQ425 as Program<AnchorDiceGameQ425>;

  const MSG = Uint8Array.from(Buffer.from("1337", "hex"));
  let house = new Keypair();
  let player = new Keypair();
  let seed = new BN(randomBytes(16));
  let vault = PublicKey.findProgramAddressSync([Buffer.from("vault"), house.publicKey.toBuffer()], program.programId)[0];
  let bet = PublicKey.findProgramAddressSync([Buffer.from("bet"), vault.toBuffer(), seed.toBuffer("le", 16)], program.programId)[0];
  let signature: Uint8Array;

  before(async () => {
    await Promise.all([house, player].map(async (k) => {
      return await provider.connection.requestAirdrop(k.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    }));
  });

  it("Initialize", async () => {
    const amount = new BN(LAMPORTS_PER_SOL);

    await program.methods.initialize(amount)
      .accounts({
        house: house.publicKey,
        // @ts-ignore
        vault,
        systemProgram:SystemProgram.programId 
      })
      .signers([
        house
      ])
      .rpc();

    const vaultBalance = await provider.connection.getBalance(vault);
    expect(vaultBalance).to.equal(LAMPORTS_PER_SOL);    
  });

  it("Place a bet", async () => {
    const roll = 50;
    const amount = new BN(LAMPORTS_PER_SOL / 100);
    
    await program.methods.placeBet(seed, roll, amount)
    .accounts({
      player: player.publicKey,
      house: house.publicKey,
      // @ts-ignore
      vault,
      bet,
      systemProgram:SystemProgram.programId 
    })
    .signers([
      player
    ])
    .rpc()
    .then(async () => {
      const betAccount = await program.account.bet.fetch(bet);
      expect(betAccount.player === player.publicKey, "Player should be bet creator");
    })
    .catch((e) => {
      console.log(e);
      throw Error('Expected to work');
    });

  });

  it("Resolve a bet", async () => {
    let account = await provider.connection.getAccountInfo(bet, "confirmed");
    let sig_ix = Ed25519Program.createInstructionWithPrivateKey({
      privateKey: house.secretKey,
      // Discriminator
      message: account.data.subarray(8)
    });

    const resolve_ix = await program.methods.resolveBet(Buffer.from(sig_ix.data.buffer.slice(16+32, 16+32+64))).accounts({
        player: player.publicKey,
        house: house.publicKey,
        vault,
        bet,
        instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        systemProgram:SystemProgram.programId 
      }
    )
    .signers([
      player
    ])
    .instruction();

    const tx = new Transaction().add(sig_ix).add(resolve_ix);

    try {
      await sendAndConfirmTransaction(
        program.provider.connection,
        tx,
        [player, house]
      );
    } catch (error) {
      console.error(error);
      throw (error)
    }
  });
});