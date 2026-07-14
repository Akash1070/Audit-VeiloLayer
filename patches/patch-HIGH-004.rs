// =========================================================================
// PATCH — HIGH-004: Cross-Namespace Nullifier Reuse
// Finding:   findings/HIGH-004-cross-namespace-nullifier-reuse.md
// Test:      tests/test-HIGH-004.rs
// PoC:       poc/poc-HIGH-004.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/swap.rs AND programs/privacy-pool/src/lib.rs
//
// ── Fix: Consolidate to a single nullifier seed prefix ──────────────────────
//
// Use a single, uniform seed prefix (e.g., b"nullifier_v4") for ALL nullifier
// marker PDA derivations. This prevents nullifier reuse across instructions.
//
// 1. In `swap.rs` (transact_swap account struct):
//
// BEFORE (vulnerable):
//   #[account(
//       init,
//       payer = relayer,
//       seeds = [b"source_nullifier_v3", source_mint.as_ref(), input_nullifier_0.as_ref()],
//       bump,
//       space = NullifierMarker::LEN
//   )]
//   pub nullifier_marker_0: Account<'info, NullifierMarker>,
//
// AFTER (patched):
//   #[account(
//       init,
//       payer = relayer,
//       seeds = [b"nullifier_v4", source_mint.as_ref(), input_nullifier_0.as_ref()],
//       bump,
//       space = NullifierMarker::LEN
//   )]
//   pub nullifier_marker_0: Account<'info, NullifierMarker>,
//
// 2. In `lib.rs` (transact account struct):
//
// BEFORE (vulnerable):
//   #[account(
//       init,
//       payer = relayer,
//       seeds = [b"nullifier_v3", mint_address.as_ref(), input_nullifier_0.as_ref()],
//       bump,
//       space = NullifierMarker::LEN
//   )]
//   pub nullifier_marker_0: Account<'info, NullifierMarker>,
//
// AFTER (patched):
//   #[account(
//       init,
//       payer = relayer,
//       seeds = [b"nullifier_v4", mint_address.as_ref(), input_nullifier_0.as_ref()],
//       bump,
//       space = NullifierMarker::LEN
//   )]
//   pub nullifier_marker_0: Account<'info, NullifierMarker>,
//
// 3. Repeat the same "nullifier_v4" prefix update for all nullifier markers
//    in the reissue path structs (JperpReissueNotes, PhoenixReissueNotes, etc.).
