# Solana Dice Game

This repo contains a simple dice betting game built with [Anchor](https://www.anchor-lang.com/) and a small React frontend using Vite.

## Prerequisites

- **Rust** and the Solana toolchain (`solana --version`)
- **Node.js** (v18 or later) and **Yarn**
- **Anchor CLI** `cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked`

Ensure the Solana CLI is configured with a keypair:

```bash
solana-keygen new --outfile ~/.config/solana/id.json
solana config set --url devnet
```

## Install Dependencies

From the repository root install Node and Rust packages:

```bash
yarn install
anchor build
cd frontend && yarn install
```

## Running Locally

1. Start a local validator and deploy the program to it:

```bash
anchor localnet
```

In a second terminal, build and deploy the program:

```bash
anchor deploy --provider.cluster localnet
```

2. Launch the React frontend pointing at the local validator:

```bash
cd frontend
yarn dev
```

You should be able to connect your wallet and place bets against the locally running program.

## Deploying to Devnet

1. Ensure your Solana CLI is targeting devnet and that your wallet has SOL:

```bash
solana config set --url devnet
solana airdrop 1
```

2. Build and deploy the program:

```bash
anchor build
anchor deploy --provider.cluster devnet
```

3. After deploying, copy the new program ID printed by the deploy command and update the `declare_id!` macro in `programs/solana-dice-game/src/lib.rs` as well as `programID` in `frontend/src/App.tsx`.

4. Finally, build and run the frontend connected to devnet:

```bash
cd frontend
yarn dev
```

The app will connect to the deployed program on devnet, allowing you to place bets with your devnet wallet.

## Deploying to Mainnet

1. Switch your Solana CLI to mainnet-beta and make sure your wallet contains SOL for transaction fees:

```bash
solana config set --url https://api.mainnet-beta.solana.com
# fund your wallet with enough SOL from an exchange or other source
```

2. Build the program and deploy it to mainnet:

```bash
anchor build
anchor deploy --provider.cluster mainnet
```

3. Update the program ID references as with devnet deployment and point the frontend to mainnet:

```bash
cd frontend
yarn dev --mode mainnet
```

After deployment your program will be live on mainnet-beta. Transactions on this network use real SOL, so proceed with caution.
