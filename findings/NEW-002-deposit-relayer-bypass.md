# NEW-002: Deposit Relayer Whitelist Bypass

**Severity:** MEDIUM  
**Impact:** Timing privacy leakage, fee front-running, and griefing via unwhitelisted deposit submissions.  
**Status:** Unpatched

---

## 1. Vulnerability Description

In `lib.rs:3522-3525`, the code checks if the relayer is whitelisted ONLY when the public amount is less than or equal to 0 (representing a withdrawal/transact path):

```rust
if public_amount <= 0 {
    require!(cfg.is_relayer(&ctx.accounts.relayer.key()), PrivacyError::RelayerNotAllowed);
}
// public_amount > 0 (deposit): Whitelist check is bypassed!
```

While deposits are permissionless to allow anyone to deposit funds, skipping the whitelisting checks completely introduces operational security issues. Specifically, `require_keys_eq!(ctx.accounts.relayer.key(), ext_data.relayer)` is NOT skipped. This means a user must sign a specific relayer into their ZK proof, but an arbitrary non-whitelisted actor can act as the relayer anyway, front-running transactions or manipulating execution.

---

## 2. Attack Vectors

1. **Rent Griefing**: Non-whitelisted relayers can spam deposits, forcing the creation of nullifier markers that occupy rent lamports paid by the target system relayers.
2. **Timing Oracle & Metadata Corruptions**: A hostile actor acting as the relayer for deposits can control inclusion timing relative to Merkle tree leaf updates, narrowing down the anonymity sets.
3. **Fee Front-running**: Since anyone can submit a deposit transaction, a malicious actor can front-run user-submitted deposit transactions with their own relayer fees, stealing the fee designated for whitelisted relayers.

---

## 3. Fix

Enforce that the submitting relayer must be whitelisted even for deposits, or explicitly separate the depositor authority from the relayer fee-recipient key to prevent fee-jacking.
