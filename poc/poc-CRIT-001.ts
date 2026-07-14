import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { Program, BN, AnchorProvider } from "@coral-xyz/anchor";
import { expect } from "chai";

// =============================================================================
// POC-CRIT-001: Nullifier Replay on Reissue Paths
// =============================================================================
export async function runCrit001NullifierReplayPoc(
  program: Program<any>,
  provider: AnchorProvider,
  reissueParams: {
    jperpSlot: PublicKey;
    mintAddress: PublicKey;
    executorTokenAccount: PublicKey;
    outputCommitment0: Uint8Array;
    outputCommitment1: Uint8Array;
    proofA: number[];
    proofB: number[];
    proofC: number[];
  }
) {
  console.log("Running PoC for CRIT-001: Nullifier Replay");

  const dummyNullifier0 = Keypair.generate().publicKey.toBytes();
  const dummyNullifier1 = Keypair.generate().publicKey.toBytes();

  const [marker0Pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("nullifier_v3"), reissueParams.mintAddress.toBuffer(), dummyNullifier0],
    program.programId
  );
  const [marker1Pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("nullifier_v3"), reissueParams.mintAddress.toBuffer(), dummyNullifier1],
    program.programId
  );

  const amount = new BN(100_000_000); // 100 USDC

  // Call 1: Succeeds (markers created)
  const tx1 = await program.methods
    .jperpReissueNotes(
      amount,
      Array.from(dummyNullifier0),
      Array.from(dummyNullifier1),
      Array.from(reissueParams.outputCommitment0),
      Array.from(reissueParams.outputCommitment1),
      reissueParams.proofA,
      reissueParams.proofB,
      reissueParams.proofC
    )
    .accounts({
      jperpSlot: reissueParams.jperpSlot,
      nullifierMarker0: marker0Pda,
      nullifierMarker1: marker1Pda,
      executorTokenAccount: reissueParams.executorTokenAccount,
    })
    .rpc();
  
  console.log("First reissue call transaction signature:", tx1);

  // Call 2: REPLAY the transaction using the exact same nullifiers.
  try {
    await program.methods
      .jperpReissueNotes(
        amount,
        Array.from(dummyNullifier0),
        Array.from(dummyNullifier1),
        Array.from(reissueParams.outputCommitment0),
        Array.from(reissueParams.outputCommitment1),
        reissueParams.proofA,
        reissueParams.proofB,
        reissueParams.proofC
      )
      .accounts({
        jperpSlot: reissueParams.jperpSlot,
        nullifierMarker0: marker0Pda,
        nullifierMarker1: marker1Pda,
        executorTokenAccount: reissueParams.executorTokenAccount,
      })
      .rpc();
    
    expect.fail("CRIT-001 Replay succeeded! Vulnerability confirmed.");
  } catch (err: any) {
    console.log("Replay rejected as expected. Error:", err.message);
    expect(err.message).to.include("custom program error: 0xbc4") // AccountAlreadyInitialized
      .or.to.include("AccountAlreadyInitialized");
  }
}
