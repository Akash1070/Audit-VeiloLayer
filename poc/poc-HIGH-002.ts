import { PublicKey } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";

// =============================================================================
// POC-HIGH-002: Position PDA Key Unbound in ZK Proof
// =============================================================================
export async function runHigh002PositionPdaUnboundPoc(
  program: Program<any>,
  openPositionParams: {
    claimant: PublicKey;
    correctPositionPdaKey: Uint8Array;
    maliciousPositionPdaKey: Uint8Array;
  }
) {
  console.log("Running PoC for HIGH-002: Position PDA Key Unbound");

  try {
    // Attempt to open position with substitute key but same proof/claimant
    // await program.methods.openPosition(openPositionParams.maliciousPositionPdaKey, ...).rpc();
    expect.fail("HIGH-002: Replayed proof with substitute position PDA key succeeded!");
  } catch (err: any) {
    expect(err.message).to.include("InvalidSwapParams");
  }
}
