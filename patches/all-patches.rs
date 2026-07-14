// =============================================================
// PATCH SCRIPT: Apply all fixes described in audit findings
// Run: cargo test -- --nocapture (after applying patches)
// =============================================================
// This file documents the exact line-level changes needed.
// Apply each diff block to the referenced source file.
// =============================================================

// ─────────────────────────────────────────────────────────────
// PATCH 1 — CRIT-001: Change init_if_needed → init
// File: programs/privacy-pool/src/lib.rs
// ─────────────────────────────────────────────────────────────

// --- JperpReissueNotes (lines 2831-2847) ---
// REMOVE:
//   init_if_needed,
// ADD:
//   init,
//
// Applies to BOTH nullifier_marker_0 AND nullifier_marker_1.
// Repeat for JperpRecoverNative (lines 2947-2963),
//            PredictionReissue  (lines 3155-3172).

// --- PhoenixReissueNotes nullifier markers (lines 2387-2404) ---
// Same change: init_if_needed → init.

// --- PhoenixReissueNotes pending_reissue (lines 2412-2424) ---
// REMOVE the init_if_needed on pending_reissue.
// CHANGE to mut (account must already exist):
//
// BEFORE:
//   #[account(
//       init_if_needed,
//       payer = relayer,
//       seeds = [b"phoenix_pending_v1", ...],
//       bump,
//       space = PhoenixPendingReissue::LEN
//   )]
//   pub pending_reissue: Box<Account<'info, PhoenixPendingReissue>>,
//
// AFTER:
//   #[account(
//       mut,
//       seeds = [b"phoenix_pending_v1", ...],
//       bump = pending_reissue.bump
//   )]
//   pub pending_reissue: Box<Account<'info, PhoenixPendingReissue>>,

// ─────────────────────────────────────────────────────────────
// PATCH 2 — CRIT-001B: Add JupiterPerpSlot reissue ceiling
// File: programs/privacy-pool/src/perps.rs (jperp_reissue_notes handler)
// ─────────────────────────────────────────────────────────────

// ADD before the token transfer in jperp_reissue_notes handler:
//
//   // Enforce executor ATA balance is sufficient and matches reissue_amount
//   let executor_ata_balance = {
//       let executor_ata_data = deserialize_token_account(
//           &ctx.accounts.executor_token_account.to_account_info()
//       )?;
//       executor_ata_data.amount
//   };
//   require!(
//       reissue_amount <= executor_ata_balance,
//       PrivacyError::JperpSlotOverdraft
//   );

// ─────────────────────────────────────────────────────────────
// PATCH 3 — CRIT-002: Eliminate relayer-float path ordering issue
// File: programs/privacy-pool/src/swap.rs
// ─────────────────────────────────────────────────────────────

// Option A (simplest): Require is_prefunded == 1 for native SOL
// In transact_swap, replace lines 390-429:
//
//   if source_is_native {
//       // PATCH: disallow relayer float path
//       require!(executor.is_prefunded == 1, PrivacyError::InvalidSwapParams);
//       require!(executor.swap_amount == swap_amount, PrivacyError::InvalidSwapParams);
//       token::sync_native(CpiContext::new(
//           ctx.accounts.token_program.to_account_info(),
//           SyncNative { account: ctx.accounts.executor_source_token.to_account_info() }
//       ))?;
//   }
//
// AND remove the post-CPI vault debit block at lines 983-993.

// ─────────────────────────────────────────────────────────────
// PATCH 4 — HIGH-001: Standard Route mint validation
// File: programs/privacy-pool/src/swap.rs  (line ~785)
// ─────────────────────────────────────────────────────────────

// ADD after `require!(remaining.len() >= 4, ...)`:
//
//   // Validate executor dest token's mint matches committed dest_mint
//   ctx.accounts.executor_dest_token.reload()?;
//   require!(
//       ctx.accounts.executor_dest_token.mint == effective_mint(&dest_mint),
//       PrivacyError::InvalidMintAddress
//   );
//   // Enforce swap_data_hash is non-zero (route was committed in ZK proof)
//   require!(
//       swap_params.swap_data_hash != [0u8; 32],
//       PrivacyError::JupiterInvalidInstruction
//   );

// ─────────────────────────────────────────────────────────────
// PATCH 5 — HIGH-002: Bind position_pda_key to claimant
// File: programs/privacy-pool/src/positions.rs (open_position handler)
// ─────────────────────────────────────────────────────────────

// ADD after ZK proof verification:
//
//   // Bind position_pda_key to the claimant committed in the ZK proof.
//   // Prevents relayer substitution of the position PDA key.
//   let expected_pda_key = PoseidonHasher::hashv(&[
//       ext_data.claimant.as_ref(),
//       withdrawal_id.as_ref(),
//   ]).map_err(|_| error!(PrivacyError::MerkleHashFailed))?;
//   require!(position_pda_key == expected_pda_key, PrivacyError::InvalidSwapParams);

// ─────────────────────────────────────────────────────────────
// PATCH 6 — HIGH-003: Fix ember_unwrap cumulative cap check
// File: programs/privacy-pool/src/phoenix.rs (phoenix_ember_unwrap handler)
// ─────────────────────────────────────────────────────────────

// REPLACE the slot cap check + increment:
//
// BEFORE:
//   require!(pending_reissue.amount <= phoenix_slot.amount, PrivacyError::SlotOverdraft);
//   // ... transfer ...
//   pending_reissue.amount += amount;
//
// AFTER:
//   let new_total = pending_reissue.amount
//       .checked_add(amount)
//       .ok_or(PrivacyError::ArithmeticOverflow)?;
//   require!(new_total <= phoenix_slot.amount, PrivacyError::SlotOverdraft);
//   // ... transfer ...
//   pending_reissue.amount = new_total;

// ─────────────────────────────────────────────────────────────
// PATCH 7 — MED-001: Fix reduce_to_field off-by-one
// Files: src/swap.rs (line 73-87) AND src/lib.rs (line 491-500)
// ─────────────────────────────────────────────────────────────

// REPLACE the comparison loop with:
//
//   let needs_reduction = bytes >= FR_MODULUS;
//   if !needs_reduction { return bytes; }
//
// This handles bytes == FR_MODULUS correctly (needs_reduction = true → reduces to 0).
// Apply to BOTH occurrences:
//   1. SwapParams::reduce_to_field in swap.rs
//   2. ExtData::reduce_to_field in lib.rs
