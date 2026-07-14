# NEW-001: Cross-Namespace Nullifier Reuse

**Severity:** MEDIUM-HIGH  
**Impact:** Double-spend across instruction types (e.g., spending nullifier via `transact_swap` and then reusing it in `transact`)  
**Status:** Unpatched

---

## 1. Vulnerability Description

The standard `transact` instruction's comment notes that nullifier markers are global, using a single prefix `nullifier_v3`:
```rust
seeds = [b"nullifier_v3", mint_address.as_ref(), input_nullifier_0.as_ref()]
```

However, the `transact_swap` instruction uses a completely different prefix/seed namespace:
```rust
seeds = [b"source_nullifier_v3", source_mint.as_ref(), input_nullifier_0.as_ref()]
```

And the reissue paths (Jperp, Phoenix, Prediction) use:
```rust
seeds = [b"nullifier_v3", mint_address.as_ref(), input_nullifier_0.as_ref()]
```

Because `transact_swap` and `transact` write to different namespaces, they resolve to entirely different PDA addresses. A nullifier marked as spent via `transact_swap` is NOT marked spent in the `transact` namespace, and vice versa.

---

## 2. Attack Scenario

1. User/Attacker calls `transact_swap` spending nullifier $N_1$. The transaction succeeds, and a nullifier marker PDA is created at `source_nullifier_v3/mint/N1`.
2. Attacker then calls standard `transact` (withdrawal) using the same nullifier $N_1$.
3. The program attempts to resolve the nullifier marker at `nullifier_v3/mint/N1`. Because it resolves to a different PDA namespace, the account is fresh (does not exist on-chain).
4. The withdrawal succeeds, allowing the attacker to double-spend the same commitment/nullifier across different instruction types.

---

## 3. Fix

Consolidate all nullifier markers into a single namespace across all instruction paths (transact, transact_swap, and reissue instructions).

```rust
// Use a single prefix (e.g., nullifier_v4) for all instructions:
seeds = [b"nullifier_v4", mint_address.as_ref(), input_nullifier_0.as_ref()]
```
