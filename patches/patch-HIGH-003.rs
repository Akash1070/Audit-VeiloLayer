// =========================================================================
// PATCH — HIGH-003: Phoenix ember_unwrap Cumulative Over-Credit
// Finding:   findings/HIGH-003-phoenix-unwrap-overcredit.md
// Test:      tests/test-HIGH-003.rs
// PoC:       poc/poc-HIGH-003.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/phoenix.rs
//
// ── Fix: Validate against post-incremented amount ──────────────────────────
//
// REPLACE the validation inside the `phoenix_ember_unwrap` handler:
//
// BEFORE (vulnerable):
//   require!(pending_reissue.amount <= phoenix_slot.amount, PrivacyError::SlotOverdraft);
//   // ... transfer ...
//   pending_reissue.amount += amount;
//
// AFTER (patched):
//   let new_total = pending_reissue.amount
//       .checked_add(amount)
//       .ok_or(PrivacyError::ArithmeticOverflow)?;
//   require!(new_total <= phoenix_slot.amount, PrivacyError::SlotOverdraft);
//   // ... transfer ...
//   pending_reissue.amount = new_total;
