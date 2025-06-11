# Solana Dice Game

This repository contains a minimal [Anchor](https://github.com/coral-xyz/anchor) program for a simple on-chain dice roll game. Users bet SOL and guess a number between 1 and 6. If the guess matches the program's pseudo‑random result, they receive a reward based on the configured percentage.

The program exposes instructions to:

- **initialize** the game state
- **place_bet** with a chosen number
- **update_reward** to change the payout percentage
- **set_pause** to pause or resume betting
- **withdraw** funds from the vault

The randomness is derived from block data and user information for demonstration purposes only and is not suitable for production use without a verifiable random oracle.
