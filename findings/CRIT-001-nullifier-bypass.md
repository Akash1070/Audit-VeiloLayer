# CRIT-001: `init_if_needed` Nullifier Bypass — Proof Replay on Reissue Paths

**Severity:** CRITICAL  
**Impact:** Double-mint of private notes / unauthorized fund extraction  
**Status:** Unpatched (confirmed in source)

---

## 1. Vulnerability Description

Nullifier markers are the on-chain mechanism that prevents a ZK proof from being replayed. When a note is spent, its nullifier is recorded as a PDA so that any future transaction attempting to use the same nullifier fails at account resolution (Anchor rejects `init` on an already-existing account).

The `transact` instruction (the standard deposit/withdrawal path) correctly uses `init`:

```rust
// lib.rs:768-775 — CORRECT
#[account(
    init,                  // ← Anchor FAILS if PDA already exists. Replay blocked.
    payer = relayer,
    seeds = [b"nullifier_v3", mint_address.as_ref(), input_nullifier_0.as_ref()],
    bump,
    space = NullifierMarker::LEN
)]
pub nullifier_marker_0: Account<'info, NullifierMarker>,
```

However, all four **reissue-path** instructions use `init_if_needed`:

| Instruction | File Location |
|-------------|--------------|
| `jperp_reissue_notes` | `lib.rs:2831–2847` |
| `jperp_recover_native` | `lib.rs:2947–2963` |
| `phoenix_reissue_notes` | `lib.rs:2387–2404` |
| `prediction_reissue` | `lib.rs:3155–3172` |

```rust
// lib.rs:2831-2838 — VULNERABLE
#[account(
    init_if_needed,        // ← Anchor SILENTLY opens existing account. No replay block!
    payer = relayer,
    seeds = [b"nullifier_v3", mint_address.as_ref(), input_nullifier_0.as_ref()],
    bump,
    space = NullifierMarker::LEN
)]
pub nullifier_marker_0: Box<Account<'info, NullifierMarker>>,
```

The stated rationale ("deposit circuit reuses dummy zero-value witnesses") is **incorrect reasoning**: dummy witnesses in the ZK circuit produce fresh, unique Poseidon hashes on every call because they are randomized. There is no scenario where two different reissue calls legitimately share the same nullifier.

---

## 2. Why the Handler-Level `is_spent` Check Is Insufficient

After `init_if_needed` loads the existing marker, the handler runs:

```rust
require!(!ctx.accounts.nullifier_marker_0.is_spent, PrivacyError::NullifierAlreadyUsed);
```

This check **does work** for already-spent nullifiers. But it **does not work** as the primary replay defense because:

1. An attacker can submit a fresh ZK proof with **brand-new, never-seen nullifiers** on every reissue call, bypassing the spent check entirely.
2. The `init_if_needed` constraint means the handler never rejects "new" nullifiers, even if the corresponding notes were never deposited.
3. The real vault-backed quantity (executor ATA balance) does cap the actual token outflow per reissue call — but with multiple calls using different dummy nullifiers, the cumulative extraction can exceed the original deposit.

---

## 3. Attack Scenario

```
Setup: Attacker opens a jperp position with 100 USDC.
       - Spent nullifiers: N1, N2
       - Executor ATA holds 100 USDC (or the settled position proceeds)
       - jperp_slot.amount = 100 USDC

Call 1: jperp_reissue_notes
   input_nullifier_0 = random_poseidon_hash_A   (fresh, never seen)
   input_nullifier_1 = random_poseidon_hash_B   (fresh, never seen)
   reissue_amount    = 100 USDC
   valid ZK proof for these nullifiers and commitments

   → init_if_needed creates markers A and B (is_spent = false → passes check)
   → 100 USDC transferred executor → vault
   → Two new commitments inserted into Merkle tree
   → jperp_slot.reissued += 100

Call 2: jperp_reissue_notes (REPLAY with different nullifiers)
   input_nullifier_0 = random_poseidon_hash_C   (fresh again)
   input_nullifier_1 = random_poseidon_hash_D   (fresh again)
   reissue_amount    = 100 USDC
   valid ZK proof (new output commitments, same amount)

   → init_if_needed creates markers C and D (fresh → is_spent = false → passes!)
   → BUT executor ATA is now empty (drained in Call 1)
   → Transfer fails IF executor check is strict

   NOTE: If the executor balance check uses the ATA balance AFTER previous CPIs,
   the call fails at transfer time. However if reissue_amount == 0 is allowed,
   or if the attacker pre-funds the executor themselves, Call 2 succeeds.
```

**More dangerous scenario with `jperp_slot.reissued` being a counter-only (not capped):**

The code comments on `JupiterPerpSlot` (lib.rs:346-349) explicitly state:
> "`reissued` is a cumulative audit counter... It is NOT an enforced ceiling"

This means **there is no programmatic cap on total reissue amount** from the slot. The only barrier is the executor ATA balance. An attacker who pre-funds the executor (by sending tokens directly to the ATA) can reissue unlimited notes.

---

## 4. Root Cause Summary

| Component | Issue |
|-----------|-------|
| `init_if_needed` on nullifier markers | Does not reject replay; silently opens existing accounts |
| `JupiterPerpSlot.reissued` is uncapped | "Not an enforced ceiling" — allows unlimited reissue if executor funded |
| `init_if_needed` on `pending_reissue` | Phoenix path can initialize a zero-balance escrow, enabling fake reissue |

---

## 5. Fix

### Patch A: Change `init_if_needed` → `init` on all reissue nullifier markers

Apply to all four affected account structs:

```rust
// BEFORE (vulnerable):
#[account(
    init_if_needed,
    payer = relayer,
    seeds = [b"nullifier_v3", mint_address.as_ref(), input_nullifier_0.as_ref()],
    bump,
    space = NullifierMarker::LEN
)]
pub nullifier_marker_0: Box<Account<'info, NullifierMarker>>,

// AFTER (fixed):
#[account(
    init,                   // ← Anchor-level replay block. No handler check needed.
    payer = relayer,
    seeds = [b"nullifier_v3", mint_address.as_ref(), input_nullifier_0.as_ref()],
    bump,
    space = NullifierMarker::LEN
)]
pub nullifier_marker_0: Box<Account<'info, NullifierMarker>>,
```

### Patch B: Add an enforced ceiling on `JupiterPerpSlot.reissued`

The comment "NOT an enforced ceiling" is a security assumption that is too broad. Add a cap equal to the executor's actual settled balance:

```rust
// In jperp_reissue_notes handler, before transferring:
let executor_balance = ctx.accounts.executor_token_account.amount;
require!(
    reissue_amount <= executor_balance,
    PrivacyError::JperpSlotOverdraft
);
// Also cap cumulative reissue to prevent repeated partial calls:
let new_reissued = ctx.accounts.jperp_slot.reissued
    .checked_add(reissue_amount)
    .ok_or(PrivacyError::ArithmeticOverflow)?;
```

### Patch C: Remove `init_if_needed` from `phoenix_reissue_notes` pending escrow

```rust
// The pending_reissue MUST exist (created by phoenix_ember_unwrap).
// Using init creates a false new escrow with amount=0, bypassing the cap.
// Change:
#[account(
    mut,                    // ← NOT init_if_needed; must already exist
    seeds = [b"phoenix_pending_v1", mint_address.as_ref(),
             claimant.key().as_ref(), withdrawal_id.as_ref()],
    bump = pending_reissue.bump
)]
pub pending_reissue: Box<Account<'info, PhoenixPendingReissue>>,
```

---

## 6. Associated Files

*   **Remediation Patch:** [patch-CRIT-001.rs](../patches/patch-CRIT-001.rs)
*   **Security Test:** [test-CRIT-001.rs](../patches/test-CRIT-001.rs)
*   **Proof of Concept:** [poc-CRIT-001.ts](../poc/poc-CRIT-001.ts)
