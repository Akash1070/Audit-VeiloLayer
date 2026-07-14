// =========================================================================
// PATCH — MED-002: Deposit Relayer Whitelist Bypass
// Finding:   findings/MED-002-deposit-relayer-bypass.md
// Test:      tests/test-MED-002.rs
// PoC:       poc/poc-MED-002.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/lib.rs
//
// ── Fix: Validate relayer whitelist for all operations ─────────────────────
//
// Require the relayer to be whitelisted for deposits as well, or separate
// the transaction signer authority from the fee-recipient relayer key.
//
// In `deposit` / `transact` instruction handler:
//
// BEFORE (vulnerable):
//   if public_amount <= 0 {
//       require!(cfg.is_relayer(&ctx.accounts.relayer.key()), PrivacyError::RelayerNotAllowed);
//   }
//
// AFTER (patched):
//   // Enforce whitelist check regardless of public_amount direction
//   require!(
//       cfg.is_relayer(&ctx.accounts.relayer.key()), 
//       PrivacyError::RelayerNotAllowed
//   );
