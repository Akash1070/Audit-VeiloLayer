// =========================================================================
// PATCH — MED-003: Executor PDA Rent-Griefing (DoS)
// Finding:   findings/MED-003-executor-pda-dos.md
// Test:      tests/test-MED-003.rs
// PoC:       poc/poc-MED-003.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/swap.rs
//
// ── Fix 1: Use init_if_needed for the Executor PDA ─────────────────────────
//
// Change `init` to `init_if_needed` on the `executor` account definition:
//
// BEFORE (vulnerable):
//   #[account(
//       init,
//       payer = relayer,
//       seeds = [b"executor_v1", relayer.key().as_ref(), nullifier.as_ref()],
//       bump,
//       space = Executor::LEN
//   )]
//   pub executor: Account<'info, Executor>,
//
// AFTER (patched):
//   #[account(
//       init_if_needed,
//       payer = relayer,
//       seeds = [b"executor_v1", relayer.key().as_ref(), nullifier.as_ref()],
//       bump,
//       space = Executor::LEN
//   )]
//   pub executor: Account<'info, Executor>,
//
// ── Fix 2: Explicitly close the executor account on completion ──────────────
//
// In the handler, replace the manual lamport draining with Anchor's closing helper:
//
// BEFORE (vulnerable):
//   let executor_lamports = executor.to_account_info().lamports();
//   **executor.to_account_info().try_borrow_mut_lamports()? = 0;
//   **ctx.accounts.relayer.to_account_info().try_borrow_mut_lamports()? += executor_lamports;
//
// AFTER (patched):
//   // Cleanly close the account so its state/discriminator is erased
//   executor.close(ctx.accounts.relayer.to_account_info())?;
