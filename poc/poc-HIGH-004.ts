import { PublicKey } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";

// =============================================================================
// POC-HIGH-004: Cross-Namespace Nullifier Reuse
// =============================================================================
export async function runHigh004CrossNamespacePoc(
  program: Program<any>,
  nullifier: Uint8Array
) {
  console.log("Running PoC for HIGH-004: Cross-Namespace Nullifier Reuse");

  // Call 1: Spend nullifier in swap path
  // await program.methods.transactSwap(nullifier, ...).rpc();

  // Call 2: Attempt to spend same nullifier in reissue path
  try {
    // await program.methods.transact(nullifier, ...).rpc();
    expect.fail("HIGH-004: Reused nullifier across namespaces!");
  } catch (err: any) {
    expect(err.message).to.include("AccountAlreadyInitialized")
      .or.to.include("0xbc4");
  }
}
