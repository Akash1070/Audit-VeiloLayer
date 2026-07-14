# HIGH-003: Phoenix `ember_unwrap` Cumulative Over-Credit Before Slot Cap Check

**Severity:** HIGH  
**Impact:** Phantom USDC minted/credited beyond the designated Phoenix slot withdrawal cap  
**Status:** Unpatched

---

## 1. Vulnerability Description

In `phoenix.rs` inside the `phoenix_ember_unwrap` handler, the slot cap check is performed on the historical amount *before* updating it with the newly unwrapped amount:
```rust
// Conceptual handler logic:
require!(pending_reissue.amount <= phoenix_slot.amount, SlotOverdraft);
// ...transfer USDC to vault...
pending_reissue.amount += received_amount;   // ← added AFTER check
```

Because the comparison uses the pre-incremented `pending_reissue.amount`, an attacker can call `phoenix_ember_unwrap` multiple times in a single transaction or sequence of transactions. If each call is for an amount just under the cap limit, every single call will pass the cap check. This allows them to accumulate far more total reissued USDC than the allowed `phoenix_slot.amount` limit.

---

## 2. Attack Scenario

1. The Phoenix slot has a withdrawal cap configured at 100 USDC.
2. The attacker calls `phoenix_ember_unwrap` with an unwrap amount of 90 USDC.
   * `pending_reissue.amount` is currently 0.
   * Check: `0 <= 100` passes.
   * `pending_reissue.amount` is incremented to 90.
3. The attacker calls `phoenix_ember_unwrap` again with another 90 USDC in the same transaction or block.
   * `pending_reissue.amount` is currently 90.
   * Check: `90 <= 100` passes!
   * `pending_reissue.amount` is incremented to 180 USDC (violating the 100 USDC slot cap).
4. The attacker successfully over-credits their position, minting phantom USDC commitments.

---

## 3. Fix & Associated Files

Ensure that the cap check is evaluated against the *post-incremented* total amount:

```rust
let new_total = pending_reissue.amount
    .checked_add(received_amount)
    .ok_or(PrivacyError::ArithmeticOverflow)?;
require!(new_total <= phoenix_slot.amount, PrivacyError::SlotOverdraft);
pending_reissue.amount = new_total;
```

*   **Remediation Patch:** [patch-HIGH-003.rs](../patches/patch-HIGH-003.rs)
*   **Security Test:** [test-HIGH-003.rs](../patches/test-HIGH-003.rs)
*   **Proof of Concept:** [poc-HIGH-003.ts](../poc/poc-HIGH-003.ts)
