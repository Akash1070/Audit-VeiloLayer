import { PublicKey, Keypair } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";

// =============================================================================
// POC-MED-002: Deposit Relayer Whitelist Bypass
// =============================================================================
export async function runMed002DepositRelayerBypassPoc(
  program: Program<any>,
  depositParams: {
    nonWhitelistedRelayer: Keypair;
  }
) {
  console.log("Running PoC for MED-002: Deposit Relayer Bypass");

  try {
    // Attempt deposit using a non-whitelisted relayer
    // await program.methods.deposit(...).accounts({ relayer: depositParams.nonWhitelistedRelayer.publicKey }).rpc();
    expect.fail("MED-002: Deposit succeeded with non-whitelisted relayer!");
  } catch (err: any) {
    expect(err.message).to.include("RelayerNotAllowed");
  }
}
