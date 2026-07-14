import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";

// =============================================================================
// POC-HIGH-003: Phoenix ember_unwrap Cumulative Over-Credit
// =============================================================================
export async function runHigh003PhoenixOvercreditPoc(program: Program<any>) {
  console.log("Running PoC for HIGH-003: Phoenix Cumulative Over-Credit");

  // Call 1: unwrap 60 USDC (cap is 100)
  // await program.methods.phoenixEmberUnwrap(new BN(60_000_000)).rpc();
  
  // Call 2: unwrap 60 USDC (cumulative 120, exceeds 100)
  try {
    // await program.methods.phoenixEmberUnwrap(new BN(60_000_000)).rpc();
    expect.fail("HIGH-003: Exceeded Phoenix slot cap!");
  } catch (err: any) {
    expect(err.message).to.include("SlotOverdraft");
  }
}
