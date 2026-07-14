# NEW-003: `stage_swap_legs` Buffer Linkage DoS

**Severity:** LOW  
**Impact:** Denial of Service (DoS) preventing users from submitting valid swap transactions.  
**Status:** Unpatched

---

## 1. Vulnerability Description

The `stage_swap_legs` instruction stores a serialized Jupiter route inside a buffer PDA. The PDA is seeded by `[relayer, input_nullifier_0]`. When `open_position` is called, the handler reads from this buffer and verifies that `sha256(legs_bytes) == swap_params.swap_data_hash`.

While this verification is cryptographically correct, the linkage is vulnerable to a state-corruption race condition. Because `close_swap_legs` can be called independently, a malicious or malfunctioning relayer can clear or overwrite the staged route buffer *between* the time the user generates their ZK proof and the time the transaction is actually processed.

---

## 2. Impact

If a relayer or an attacker front-runs the `open_position` instruction with a `close_swap_legs` call (using the same nullifier seed), the staged buffer is closed, causing the subsequent `open_position` to fail with a missing account error. The user's ZK proof is invalidated, and they must generate a brand new proof with a different nullifier to try again.

---

## 3. Fix

Integrate the routing data into the instruction transaction directly or restrict the closing of swap legs buffers until the swap execution has completely finished within the same atomic transaction block.
