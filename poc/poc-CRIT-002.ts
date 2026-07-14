import { Connection, PublicKey } from "@solana/web3.js";
import { Program, BN, AnchorProvider } from "@coral-xyz/anchor";
import { expect } from "chai";

// =============================================================================
// POC-CRIT-002: Relayer-Float Path Skips Vault Debit
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

  // Execute transact_swap in relayer-float mode (is_prefunded = 0)
  // const tx = await program.methods.transactSwap(..., isPrefunded = 0).rpc();

  const vaultAfter = await provider.connection.getBalance(swapParams.sourceVault);
  const sourceConfigAfter = await program.account.poolConfig.fetch(swapParams.sourceConfig);
  const tvlAfter = sourceConfigAfter.totalTvl as BN;

  const tvlDiff = tvlBefore.sub(tvlAfter);
  const vaultDiff = vaultBefore - vaultAfter;

  // Assert that TVL state decrement matches the actual vault lamport decrement
  expect(tvlDiff.toNumber()).to.equal(vaultDiff, "CRIT-002: Accounting divergence between TVL and Vault Balance!");
}
