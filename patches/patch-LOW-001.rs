// =========================================================================
// PATCH — LOW-001: stage_swap_legs Buffer Linkage DoS
// Finding:   findings/LOW-001-stage-swap-legs-dos.md
// Test:      tests/test-LOW-001.rs
// PoC:       poc/poc-LOW-001.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/swap.rs
//
// ── Fix: Restrict closing of the legs buffer to the executor authority ─────
//
// Ensure that whitelisted relayers cannot close or overwrite the legs buffer
// unless they are executing the corresponding swap in the same transaction, or
// restrict the closing logic to the transaction initializer (user).
//
// In `close_swap_legs` instruction handler:
//
// BEFORE (vulnerable):
//   pub fn close_swap_legs(ctx: Context<CloseSwapLegs>) -> Result<()> {
//       // Closes the account immediately and sends lamports to relayer
//       Ok(())
//   }
//
// AFTER (patched):
//   pub fn close_swap_legs(ctx: Context<CloseSwapLegs>) -> Result<()> {
//       // Validate that the buffer is only closed after the swap has completed
//       // or require the signer to match the claimant signature.
//       require!(
//           ctx.accounts.signer.key() == ctx.accounts.legs_buffer.claimant,
//           PrivacyError::UnauthorizedSigner
//       );
//       Ok(())
//   }
