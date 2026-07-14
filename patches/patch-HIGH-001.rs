// =========================================================================
// PATCH — HIGH-001: Jupiter Route Injection (No Intermediate Mint Check)
// Finding:   findings/HIGH-001-jupiter-route-injection.md
// Test:      tests/test-HIGH-001.rs
// PoC:       poc/poc-HIGH-001.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/swap.rs
//
// ── Fix: Enforce destination mint and non-zero route commitment ─────────────
//
// ADD after `require!(remaining.len() >= 4, ...)` in the Jupiter route handler:
//
//   // Validate that executor destination token account mint matches committed dest_mint.
//   // Prevents attacker from redirecting swap proceeds to a foreign token.
//   ctx.accounts.executor_dest_token.reload()?;
//   require!(
//       ctx.accounts.executor_dest_token.mint == effective_mint(&dest_mint),
//       PrivacyError::InvalidMintAddress
//   );
//
//   // Enforce swap_data_hash is non-zero: user must have committed a specific route
//   // in their ZK proof, preventing the relayer from injecting an arbitrary route.
//   require!(
//       swap_params.swap_data_hash != [0u8; 32],
//       PrivacyError::JupiterInvalidInstruction
//   );
//
// ── Additional hardening (recommended) ─────────────────────────────────────
//
// Verify the hash of the actual remaining route accounts matches swap_data_hash:
//
//   let route_bytes: Vec<u8> = remaining
//       .iter()
//       .flat_map(|ai| ai.key.to_bytes())
//       .collect();
//   let actual_hash = solana_sha256_hasher::hash(&route_bytes).to_bytes();
//   require!(
//       actual_hash == swap_params.swap_data_hash,
//       PrivacyError::JupiterInvalidInstruction
//   );
