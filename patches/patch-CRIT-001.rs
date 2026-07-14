// =========================================================================
// PATCH — CRIT-001: init_if_needed Nullifier Bypass on Reissue Paths
// Finding:   findings/CRIT-001-nullifier-bypass.md
// Test:      tests/test-CRIT-001.rs
// PoC:       poc/poc-CRIT-001.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/lib.rs
//
// ── Part A: Change init_if_needed → init on ALL four reissue paths ─────────
//
// AFFECTS:
//   • JperpReissueNotes    (lines 2831–2847) — nullifier_marker_0 and _1
//   • JperpRecoverNative   (lines 2947–2963) — nullifier_marker_0 and _1
//   • PhoenixReissueNotes  (lines 2387–2404) — nullifier_marker_0 and _1
//   • PredictionReissue    (lines 3155–3172) — nullifier_marker_0 and _1
//
// BEFORE (vulnerable):
//   #[account(
//       init_if_needed,              // ← silently opens already-initialized accounts
//       payer = relayer,
//       seeds = [b"nullifier_v3", mint_address.as_ref(), input_nullifier_0.as_ref()],
//       bump,
//       space = NullifierMarker::LEN
//   )]
//   pub nullifier_marker_0: Box<Account<'info, NullifierMarker>>,
//
// AFTER (patched):
//   #[account(
//       init,                        // ← Anchor fails hard if PDA already exists
//       payer = relayer,
//       seeds = [b"nullifier_v3", mint_address.as_ref(), input_nullifier_0.as_ref()],
//       bump,
//       space = NullifierMarker::LEN
//   )]
//   pub nullifier_marker_0: Box<Account<'info, NullifierMarker>>,
//
// Apply identically to nullifier_marker_1 in each instruction.
//
// ── Part B: Fix PhoenixReissueNotes pending_reissue — must EXIST, not be CREATED
//
// BEFORE (vulnerable):
//   #[account(
//       init_if_needed,
//       payer = relayer,
//       seeds = [b"phoenix_pending_v1", mint_address.as_ref(),
//                claimant.key().as_ref(), withdrawal_id.as_ref()],
//       bump,
//       space = PhoenixPendingReissue::LEN
//   )]
//   pub pending_reissue: Box<Account<'info, PhoenixPendingReissue>>,
//
// AFTER (patched):
//   #[account(
//       mut,                         // ← must already exist (created by phoenix_ember_unwrap)
//       seeds = [b"phoenix_pending_v1", mint_address.as_ref(),
//                claimant.key().as_ref(), withdrawal_id.as_ref()],
//       bump = pending_reissue.bump
//   )]
//   pub pending_reissue: Box<Account<'info, PhoenixPendingReissue>>,
//
// ── Part C: Add enforced reissue ceiling in jperp_reissue_notes handler ────
//
// FILE: programs/privacy-pool/src/perps.rs
//
// ADD before the token transfer in jperp_reissue_notes handler:
//
//   let new_reissued = ctx.accounts.jperp_slot.reissued
//       .checked_add(reissue_amount)
//       .ok_or(PrivacyError::ArithmeticOverflow)?;
//   require!(
//       new_reissued <= ctx.accounts.jperp_slot.amount,
//       PrivacyError::JperpSlotOverdraft
//   );
//   ctx.accounts.jperp_slot.reissued = new_reissued;
