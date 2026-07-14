# HIGH-005: `fund_native_open_position` Missing Atomic Pairing Validation

**Severity:** HIGH  
**Impact:** Unauthorized SOL drain from the position pool vault by a malicious or compromised relayer.  
**Status:** Unpatched

---

## 1. Vulnerability Description

To prevent funds from being drained from the vault without executing the corresponding state transitions, the program uses Sysvar-based instruction pairing. For instance, `fund_native_source` validates that the next instruction in the transaction is `transact_swap`:

```rust
// swap.rs:160-189 (fund_native_source) — CORRECT
let next_ix = load_instruction_at_checked(current_idx + 1, &ix_sysvar)?;
require!(next_ix.data[..8] == TRANSACT_SWAP_DISCRIMINATOR, ...);
```

However, the `fund_native_open_position` instruction (lib.rs:3808-3815, positions.rs) lacks any atomic pairing checks. It permits a relayer to call funding in isolation without forcing the execution of `open_position` in the same transaction block.

---

## 2. Attack Scenario

1. A malicious or compromised whitelisted relayer calls `fund_native_open_position(swap_amount = 50 SOL)`.
2. The vault is debited 50 SOL, and the funds are transferred to the executor PDA's WSOL account.
3. The relayer does NOT append the `open_position` instruction to the transaction.
4. Because the instruction is missing, the position state is never updated, but the 50 SOL remains sitting in the executor's WSOL account.
5. The relayer invokes their own program to close the executor's WSOL account and claim the 50 SOL.

---

## 3. Fix

Add instruction-sysvar checks to `fund_native_open_position` to ensure `open_position` is called next:

```rust
let ix_sysvar = &ctx.accounts.instructions;
let current_idx = load_current_index_checked(ix_sysvar)? as usize;
let hash = solana_sha256_hasher::hash(b"global:open_position");
let disc: [u8; 8] = hash.to_bytes()[..8].try_into()?;
let next_ix = load_instruction_at_checked(current_idx + 1, ix_sysvar)
    .map_err(|_| error!(PrivacyError::MissingOpenPositionInstruction))?;
require!(
    next_ix.program_id == crate::id()
    && next_ix.data.len() >= 8
    && next_ix.data[..8] == disc,
    PrivacyError::MissingOpenPositionInstruction
);
```

---

## 4. Associated Files

*   **Remediation Patch:** [patch-HIGH-005.rs](../patches/patch-HIGH-005.rs)
*   **Security Test:** [test-HIGH-005.rs](../patches/test-HIGH-005.rs)
*   **Proof of Concept:** [poc-HIGH-005.ts](../poc/poc-HIGH-005.ts)
