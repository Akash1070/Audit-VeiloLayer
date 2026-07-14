// =========================================================================
// PATCH — MED-001: reduce_to_field Off-By-One on FR_MODULUS Equality
// Finding:   findings/MED-001-reduce-to-field-off-by-one.md
// Test:      tests/test-MED-001.rs
// PoC:       poc/poc-MED-001.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/swap.rs AND programs/privacy-pool/src/lib.rs
//
// ── Fix: Use direct relational comparison ──────────────────────────────────
//
// REPLACE the comparison loop with a direct `>=` check.
//
// 1. In `SwapParams::reduce_to_field` (swap.rs):
//
// BEFORE (vulnerable):
//   let mut needs_reduction = false;
//   for i in 0..32 {
//       if bytes[i] < FR_MODULUS[i] { break; }
//       if bytes[i] > FR_MODULUS[i] { needs_reduction = true; break; }
//   }
//   if !needs_reduction { return bytes; }
//   // perform reduction ...
//
// AFTER (patched):
//   let needs_reduction = bytes >= FR_MODULUS;
//   if !needs_reduction { return bytes; }
//   // perform reduction ...
//
// 2. Repeat the exact same replacement in `ExtData::reduce_to_field` (lib.rs).
