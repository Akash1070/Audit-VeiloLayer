// =========================================================================
// PATCH — CRIT-002: Relayer-Float Path Skips Vault Debit
// Finding:   findings/CRIT-002-relayer-float-vault-debit.md
// Test:      tests/test-CRIT-002.rs
// PoC:       poc/poc-CRIT-002.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/swap.rs
//
// ── Option A (Recommended — Simplest): Disallow relayer-float for native SOL
//
// REPLACE lines 390–429 in transact_swap:
//
// BEFORE (vulnerable):
//   if source_is_native {
//       if executor.is_prefunded == 0 {
//           // Relayer funds executor from their own balance
//           anchor_lang::system_program::transfer(
//               CpiContext::new(system_program.clone(), Transfer {
//                   from: ctx.accounts.relayer.to_account_info(),
//                   to: ctx.accounts.executor_source_token.to_account_info(),
//               }),
//               swap_amount
//           )?;
//       }
//       // ... sync native, TVL decrement at line 464 ...
//   }
//   // ... post-CPI vault debit at lines 983-993 (ONLY in float path) ...
//
// AFTER (patched — Option A):
//   if source_is_native {
//       // Disallow relayer-float path; executor MUST be pre-funded
//       require!(executor.is_prefunded == 1, PrivacyError::InvalidSwapParams);
//       require!(executor.swap_amount == swap_amount, PrivacyError::InvalidSwapParams);
//       token::sync_native(CpiContext::new(
//           ctx.accounts.token_program.to_account_info(),
//           SyncNative { account: ctx.accounts.executor_source_token.to_account_info() },
//       ))?;
//   }
//   // Remove post-CPI vault debit block at lines 983-993.
//
// ── Option B (Alternative): Debit vault BEFORE swap executes ──────────────
//
// If retaining the float path is required, move vault debit to BEFORE the CPI:
//
//   if source_is_native && executor.is_prefunded == 0 {
//       // Debit vault NOW, before TVL decrement and CPI
//       let vault_lamports = ctx.accounts.source_vault.lamports();
//       require!(vault_lamports >= swap_amount, PrivacyError::InsufficientVaultBalance);
//       **ctx.accounts.source_vault.try_borrow_mut_lamports()? -= swap_amount;
//       **ctx.accounts.relayer.to_account_info().try_borrow_mut_lamports()? += swap_amount;
//   }
//   // Now decrement TVL (accounting now consistent with vault state)
//   ctx.accounts.source_config.total_tvl = ctx.accounts.source_config.total_tvl
//       .checked_sub(swap_amount)
//       .ok_or(PrivacyError::ArithmeticOverflow)?;
