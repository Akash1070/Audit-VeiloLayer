// =========================================================================
// PATCH — HIGH-002: Position PDA Key Unbound in ZK Proof
// Finding:   findings/HIGH-002-position-pda-key-unbound.md
// Test:      tests/test-HIGH-002.rs
// PoC:       poc/poc-HIGH-002.ts
// =========================================================================
//
// FILE: programs/privacy-pool/src/positions.rs
//
// ── Fix: Derive and enforce position_pda_key from claimant + withdrawal_id ──
//
// ADD immediately after ZK proof verification succeeds in open_position handler:
//
//   // Bind position_pda_key to the claimant committed inside the ZK proof.
//   // Without this, any relayer can supply an arbitrary position_pda_key
//   // pointing to a PDA they control, hijacking the opened position.
//   let expected_pda_key = PoseidonHasher::hashv(&[
//       ext_data.claimant.as_ref(),
//       withdrawal_id.as_ref(),
//   ])
//   .map_err(|_| error!(PrivacyError::MerkleHashFailed))?;
//
//   require!(
//       position_pda_key == expected_pda_key,
//       PrivacyError::InvalidSwapParams
//   );
//
// ── Alternative (if Poseidon not available off-circuit) ────────────────────
//
// Derive the expected key using SHA-256 and compare:
//
//   let raw = [ext_data.claimant.as_ref(), withdrawal_id.as_ref()].concat();
//   let expected = solana_sha256_hasher::hash(&raw).to_bytes();
//   require!(
//       position_pda_key == expected,
//       PrivacyError::InvalidSwapParams
//   );
