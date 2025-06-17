import * as anchor from "@project-serum/anchor";
import { useEffect, useState } from "react";
import { AnchorProvider, Program, web3 } from "@project-serum/anchor";
import { Buffer } from "buffer";

// Ensure Buffer is available in the browser
(window as any).Buffer = Buffer;
import idl from "./idl/solana_dice_game.json";

const { SystemProgram } = web3;
const programID = new web3.PublicKey(
  "BkiKwAoQEogZtundzzn5buhLfntYdoiFABur66buDXMM"
);

const network = "https://api.devnet.solana.com";

function getProvider() {
  const connection = new web3.Connection(network, "processed");
  const provider = new AnchorProvider(connection, (window as any).solana, {
    preflightCommitment: "processed",
  });
  return provider;
}

export default function App() {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [chosenNumber, setChosenNumber] = useState(1);
  const [betAmount, setBetAmount] = useState(0.1);

  useEffect(() => {
    const provider = (window as any).solana;
    if (provider?.isPhantom) {
      provider.connect({ onlyIfTrusted: true }).then((res: any) => {
        setWalletAddress(res.publicKey.toString());
      });
    }
  }, []);

  const connectWallet = async () => {
    const { solana } = window as any;
    if (solana) {
      const response = await solana.connect();
      setWalletAddress(response.publicKey.toString());
    }
  };

  const placeBet = async () => {
    const provider = getProvider();
    const program = new Program(idl as any, programID, provider);

    const [gameConfigPDA] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("game-config")],
      programID
    );

    const lamports = betAmount * web3.LAMPORTS_PER_SOL;

    await program.methods
      .placeBet(chosenNumber, new anchor.BN(lamports))
      .accounts({
        gameConfig: gameConfigPDA,
        player: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  };

  return (
    <div style={{ padding: "2rem" }}>
      <h1>Solana Dice Game</h1>
      {walletAddress ? (
        <p>Wallet: {walletAddress}</p>
      ) : (
        <button onClick={connectWallet}>Connect Wallet</button>
      )}
      <div style={{ marginTop: "1rem" }}>
        <label>
          Choose Number (1-6):
          <input
            type="number"
            min="1"
            max="6"
            value={chosenNumber}
            onChange={(e) => setChosenNumber(Number(e.target.value))}
          />
        </label>
      </div>
      <div style={{ marginTop: "1rem" }}>
        <label>
          Bet Amount (SOL):
          <input
            type="number"
            step="0.01"
            value={betAmount}
            onChange={(e) => setBetAmount(Number(e.target.value))}
          />
        </label>
      </div>
      <button style={{ marginTop: "1rem" }} onClick={placeBet} disabled={!walletAddress}>
        Place Bet
      </button>
    </div>
  );
}
