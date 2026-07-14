# MED-001: `reduce_to_field` Off-By-One on `FR_MODULUS` Equality

**Severity:** MEDIUM  
**Impact:** Input equal to `FR_MODULUS` silently bypasses field reduction and produces a zero element  
**Status:** Unpatched

---

## 1. Vulnerability Description

The reduction routine `reduce_to_field` checks if the input bytes exceed `FR_MODULUS`. However, the loop structure contains an off-by-one comparison logic error:
```rust
// swap.rs:73-87 and lib.rs:491-500
let mut needs_reduction = false;
for i in 0..32 {
    if bytes[i] < FR_MODULUS[i] { break; }           // exits loop: no reduction
    if bytes[i] > FR_MODULUS[i] { needs_reduction = true; break; }
    // if equal: loop continues without setting needs_reduction
}
// When bytes == FR_MODULUS exactly: loop finishes, needs_reduction = false
// → returns FR_MODULUS, which = 0 (mod FR_MODULUS)
if !needs_reduction { return bytes; }
```

When the input is exactly equal to `FR_MODULUS`, the loop finishes with `needs_reduction = false`. The unreduced value is returned. Because `FR_MODULUS` is mathematically congruent to $0 \pmod{\text{FR\_MODULUS}}$, this results in a degenerate zero field element in ZK-SNARK verification inputs, potentially allowing malformed proofs to bypass checks.

---

## 2. Attack Scenario

1. An attacker constructs a transaction using an input value exactly equal to the field modulus `FR_MODULUS`.
2. The comparison loop fails to trigger `needs_reduction`, so the raw unreduced bytes are serialized.
3. On-chain ZK verification or hash computation interprets this value, producing a zero-value witness or swap parameters collision.
4. The attacker exploits the resulting math collision to satisfy verification checks that would otherwise fail under non-zero values.

---

## 3. Fix & Associated Files

Replace the custom loop with a direct comparison:

```rust
let needs_reduction = bytes >= FR_MODULUS;
if !needs_reduction { return bytes; }
```

Apply this fix to both instances:
1. `SwapParams::reduce_to_field` in `swap.rs`
2. `ExtData::reduce_to_field` in `lib.rs`

*   **Remediation Patch:** [patch-MED-001.rs](../patches/patch-MED-001.rs)
*   **Security Test:** [test-MED-001.rs](../patches/test-MED-001.rs)
*   **Proof of Concept:** [poc-MED-001.ts](../poc/poc-MED-001.ts)
