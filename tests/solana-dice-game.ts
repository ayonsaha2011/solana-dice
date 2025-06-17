import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolanaDiceGame } from "../target/types/solana_dice_game.js";
import {
  PublicKey,
  SystemProgram,
  Keypair,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { assert } from "chai";

describe("solana-dice-game", () => {
  // Configure the provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaDiceGame as Program<SolanaDiceGame>;
  const admin = provider.wallet;
  const player = Keypair.generate();

  let gameConfigPDA: PublicKey;

  before(async () => {
    // Initialize game
    [gameConfigPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("game-config")],
      program.programId
    );

    await program.methods
      .initializeGame(500) // 5x reward
      .accounts({
        gameConfig: gameConfigPDA,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Fund player
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        player.publicKey,
        1 * LAMPORTS_PER_SOL
      ),
      "confirmed"
    );
  });

  it("should place a bet", async () => {
    const betAmount = 0.1 * LAMPORTS_PER_SOL;
    const chosenNumber = 3;

    const tx = await program.methods
      .placeBet(chosenNumber, betAmount)
      .accounts({
        gameConfig: gameConfigPDA,
        player: player.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    assert.isString(tx);
  });

  it("should reject invalid bets", async () => {
    try {
      await program.methods
        .placeBet(0, 0.1 * LAMPORTS_PER_SOL)
        .accounts({
          gameConfig: gameConfigPDA,
          player: player.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([player])
        .rpc();
      assert.fail("Should have thrown error");
    } catch (error) {
      assert.include(error.message, "InvalidNumber");
    }
  });

  it("should allow admin functions", async () => {
    await program.methods
      .pauseGame()
      .accounts({
        gameConfig: gameConfigPDA,
        admin: admin.publicKey,
      })
      .rpc();

    let config = await program.account.gameConfig.fetch(gameConfigPDA);
    assert.isTrue(config.isPaused);

    await program.methods
      .unpauseGame()
      .accounts({
        gameConfig: gameConfigPDA,
        admin: admin.publicKey,
      })
      .rpc();

    config = await program.account.gameConfig.fetch(gameConfigPDA);
    assert.isFalse(config.isPaused);
  });
});
