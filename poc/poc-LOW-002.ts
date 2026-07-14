import { Keypair } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";

// =============================================================================
// POC-LOW-002: No Admin Rotation Timelock
// =============================================================================
export async function runLow002AdminTimelockPoc(program: Program<any>) {
  console.log("Running PoC for LOW-002: No Admin Rotation Timelock");

  const newAdmin = Keypair.generate();

  // Attempt to claim admin role immediately after proposing (or directly)
  try {
    // await program.methods.claimAdmin().rpc();
    expect.fail("LOW-002: Successfully rotated admin immediately without timelock!");
  } catch (err: any) {
    expect(err.message).to.include("TimelockNotExpired");
  }
}
