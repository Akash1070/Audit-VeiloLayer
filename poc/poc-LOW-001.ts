import { PublicKey, Keypair } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";

// =============================================================================
// POC-LOW-001: stage_swap_legs Buffer Linkage DoS
// =============================================================================
export async function runLow001StageSwapLegsPoc(
  program: Program<any>,
  swapLegsParams: {
    legsBuffer: PublicKey;
    claimant: PublicKey;
  }
) {
  console.log("Running PoC for LOW-001: stage_swap_legs DoS");

  const maliciousSigner = Keypair.generate();

  try {
    // Attempt to close swap legs buffer using unauthorized signer
    // await program.methods.closeSwapLegs().accounts({ legsBuffer: swapLegsParams.legsBuffer, signer: maliciousSigner.publicKey }).rpc();
    expect.fail("LOW-001: Unauthorized account successfully closed legs buffer!");
  } catch (err: any) {
    expect(err.message).to.include("UnauthorizedSigner");
  }
}
