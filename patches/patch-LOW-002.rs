// =========================================================================
// PATCH — LOW-002: No Admin Rotation Timelock
// Finding:   findings/LOW-002-no-timelock.md
// Test:      tests/test-LOW-002.rs
// PoC:       poc/poc-LOW-002.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/lib.rs
//
// ── Fix: Implement a two-step admin update with a timelock delay ──────────
//
// Modify the admin update mechanism to require a proposal step followed by a
// claim step after a set lock period.
//
// In `GlobalConfig`:
//
//   pub pending_admin: Pubkey,
//   pub admin_rotation_unlock_time: i64,
//
// In `propose_admin` instruction:
//
//   pub fn propose_admin(ctx: Context<GlobalConfigAdmin>, new_admin: Pubkey) -> Result<()> {
//       let cfg = &mut ctx.accounts.config;
//       cfg.pending_admin = new_admin;
//       cfg.admin_rotation_unlock_time = Clock::get()?.unix_timestamp + 259200; // 3-day timelock
//       Ok(())
//   }
//
// In `claim_admin` instruction:
//
//   pub fn claim_admin(ctx: Context<GlobalConfigPendingAdmin>) -> Result<()> {
//       let cfg = &mut ctx.accounts.config;
//       require!(
//           Clock::get()?.unix_timestamp >= cfg.admin_rotation_unlock_time,
//           PrivacyError::TimelockNotExpired
//       );
//       cfg.admin = cfg.pending_admin;
//       cfg.pending_admin = Pubkey::default();
//       Ok(())
//   }
