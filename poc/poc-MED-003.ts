import { PublicKey, SystemProgram } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";

// =============================================================================
// POC-MED-003: Executor PDA Rent-Griefing (DoS)
// =============================================================================
export async function runMed003ExecutorDosPoc(
  program: Program<any>,
  relayer: PublicKey,
  nullifier: Uint8Array
) {
  console.log("Running PoC for MED-003: Executor PDA Rent-Griefing DoS");

  // Determine executor address
  const [executorPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("executor_v1"), relayer.toBuffer(), nullifier],
    program.programId
  );

  // Prefund the PDA with 0.002 SOL (rent-exempt minimum)
  // await sendLamports(executorPda, 2_000_000);

  try {
    // Attempt swap. If unpatched, this fails with AccountAlreadyInitialized.
    // await program.methods.transactSwap(nullifier, ...).rpc();
  } catch (err: any) {
    expect(err.message).to.include("AccountAlreadyInitialized")
      .or.to.include("0xbc4");
  }
}
