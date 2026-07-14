# HIGH-002: Position PDA Key Unbound in ZK Proof — Position Hijacking

**Severity:** HIGH  
**Impact:** Attacker can hijack a victim's open position by submitting the same proof with a different `position_pda_key`  
**Status:** Unpatched

---

## 1. Vulnerability Description

`SwapPublicInputs` (swap.rs:132–141) does not include `position_pda_key`:
```rust
pub struct SwapPublicInputs {
    pub source_root: [u8; 32],
    pub swap_params_hash: [u8; 32],
    pub ext_data_hash: [u8; 32],   // includes claimant
    pub source_mint: Pubkey,
    pub dest_mint: Pubkey,
    pub input_nullifiers: [[u8; 32]; 2],
    pub output_commitments: [[u8; 32]; 2],
    pub swap_amount: u64,
    // ← NO position_pda_key
}
```

`ext_data.claimant` is included via `ext_data_hash`, which binds the ephemeral key. However, `position_pda_key` is a separate 32-byte seed passed as an instruction argument. Since it is not part of the ZK public inputs, the proof does not commit to it.

A relayer who observes a valid `open_position` call can substitute any `position_pda_key` value they control. The resulting `position_pda` PDA is owned by the attacker's keypair. When `close_position` is called, the attacker's claimant key signs — not the victim's.

## 2. Mitigating Factor

`ext_data.claimant` is in the ZK proof. The `position_pda` account has `has_one = claimant` for `close_position` (lib.rs:1422–1426):
```rust
#[account(
    mut,
    seeds = [b"position_pda_v1", position_pda_key.as_ref()],
    bump = position_pda.bump,
    close = relayer,
)]
pub position_pda: Box<Account<'info, PositionPDA>>,
```

But there is no `has_one = claimant` constraint on `open_position`'s `position_pda` creation! The `claimant` field is only stored in `position_pda` if the handler explicitly sets it. If the handler sets `position_pda.claimant = ext_data.claimant`, then the attacker's substitute `position_pda_key` would produce a PDA with the victim's claimant — defeating the hijack. If not, the hijack succeeds.

---

## 3. Attack Scenario

1. Attacker monitors the mempool for a transaction executing `open_position` containing a valid ZK proof.
2. Attacker extracts the ZK proof and `SwapPublicInputs` from the transaction.
3. Attacker constructs a new `open_position` transaction substituting their own `position_pda_key` (which they generate from a keypair they control).
4. Since `position_pda_key` is not bound to the proof, the ZK verification succeeds.
5. The program initializes the new position PDA. If `position_pda.claimant` is not strictly bound to the verified ZK claimant, the attacker can close the position later and withdraw the locked funds.

---

## 4. Fix & Associated Files

Bind `position_pda_key` to `ext_data.claimant` cryptographically or by strict derivation:

*   **Remediation Patch:** [patch-HIGH-002.rs](../patches/patch-HIGH-002.rs)
*   **Security Test:** [test-HIGH-002.rs](../patches/test-HIGH-002.rs)
*   **Proof of Concept:** [poc-HIGH-002.ts](../poc/poc-HIGH-002.ts)
