# LOW-002: No Admin Rotation Timelock

**Severity:** LOW  
**Impact:** Operational key compromise vulnerability.  
**Status:** Unpatched

---

## 1. Vulnerability Description

The `update_global_config` instruction is used to update the pool's admin parameters. It is reserved for administrative tasks and uses the standard `GlobalConfigAdmin` check:

```rust
pub fn update_global_config(_ctx: Context<GlobalConfigAdmin>) -> Result<()> {
    // Reserved for future global configuration updates
    Ok(())
}
```

The configuration is updated instantly. In the event of a key compromise of the `admin` key (or a compromise of the multisig controlling the admin key), a malicious actor can immediately change configuration parameters, authorize malicious relayers, or reconfigure pool parameters without any timelock delay.

---

## 2. Recommendation

Introduce a timelock mechanism or a multi-step admin rotation process (e.g., a two-step transfer where the new admin must claim the role after a delay) to allow the community or multi-sig guardians to intervene in the event of an emergency.

---

## 3. Associated Files

*   **Remediation Patch:** [patch-LOW-002.rs](../patches/patch-LOW-002.rs)
*   **Security Test:** [test-LOW-002.rs](../patches/test-LOW-002.rs)
*   **Proof of Concept:** [poc-LOW-002.ts](../poc/poc-LOW-002.ts)
