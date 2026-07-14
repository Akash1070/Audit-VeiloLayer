// =========================================================================
// PATCH — HIGH-005: fund_native_open_position Missing Instruction Pairing
// Finding:   findings/HIGH-005-fund-native-position-bypass.md
// Test:      tests/test-HIGH-005.rs
// PoC:       poc/poc-HIGH-005.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/lib.rs
//
// ── Fix: Add sysvar instruction pairing validation ───────────────────────
//
// Enforce that `fund_native_open_position` cannot be called in isolation
// by verifying that the subsequent instruction in the transaction is
// `open_position`.
//
// In `fund_native_open_position` handler:
//
// BEFORE (vulnerable):
//   pub fn fund_native_open_position(ctx: Context<FundNativeOpenPosition>, amount: u64) -> Result<()> {
//       // Transfers native lamports to executor account
//       Ok(())
//   }
//
// AFTER (patched):
//   pub fn fund_native_open_position(ctx: Context<FundNativeOpenPosition>, amount: u64) -> Result<()> {
//       let ix_sysvar = &ctx.accounts.instructions;
//       let current_idx = load_current_index_checked(ix_sysvar)? as usize;
//       
//       // Verify that the next instruction is open_position
//       let hash = solana_sha256_hasher::hash(b"global:open_position");
//       let disc: [u8; 8] = hash.to_bytes()[..8].try_into()?;
//       
//       let next_ix = load_instruction_at_checked(current_idx + 1, ix_sysvar)
//           .map_err(|_| error!(PrivacyError::MissingOpenPositionInstruction))?;
//           
//       require!(
//           next_ix.program_id == crate::id()
//           && next_ix.data.len() >= 8
//           && next_ix.data[..8] == disc,
//           PrivacyError::MissingOpenPositionInstruction
//       );
//       
//       // Perform native lamports transfer...
//       Ok(())
//   }
