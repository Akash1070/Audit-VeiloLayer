import { Program, BN } from "@coral-xyz/anchor";
import { expect } from "chai";

// =============================================================================
// POC-HIGH-005: fund_native_open_position Missing Atomic Pairing
// =============================================================================
export async function runHigh005FundNativeBypassPoc(program: Program<any>) {
  console.log("Running PoC for HIGH-005: fund_native_open_position bypass");

  try {
    // Call fund_native_open_position isolated from open_position
    // await program.methods.fundNativeOpenPosition(new BN(10_000_000_000)).rpc();
    expect.fail("HIGH-005: fund_native_open_position succeeded without atomic pairing!");
  } catch (err: any) {
    expect(err.message).to.include("MissingOpenPositionInstruction");
  }
}
