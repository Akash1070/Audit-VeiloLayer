# NEW-004: Executor PDA Rent-Griefing (DoS)

**Severity:** MEDIUM  
**Impact:** Permanent Denial of Service (DoS) locking specific nullifiers out of the swap loop.  
**Status:** Unpatched

---

## 1. Vulnerability Description

In `swap.rs:995-1002`, the executor PDA's lamports are zeroed and returned to the relayer after all CPI calls are completed:

```rust
let executor_lamports = executor.to_account_info().lamports();
**executor.to_account_info().try_borrow_mut_lamports()? = 0;
**ctx.accounts.relayer.to_account_info().try_borrow_mut_lamports()? = ctx.accounts.relayer
    .to_account_info()
    .lamports()
    .checked_add(executor_lamports)
    .ok_or(PrivacyError::ArithmeticOverflow)?;
```

While the lamports are zeroed (leading to garbage collection of the account at the end of transaction execution), the account itself is NOT explicitly closed via Anchor's `close` constraint, nor does it write over its own data fields.

Crucially, the executor PDA is defined with `init` (not `init_if_needed`). If an attacker sends rent-exempt lamports (e.g., 0.002 SOL) directly to the executor PDA address before a user executes their swap, the account will already exist.

---

## 2. Attack Vector

1. Attacker pre-calculates the deterministic executor PDA address for a targeted future swap nullifier.
2. Attacker sends 0.002 SOL directly to this executor PDA address.
3. The user tries to submit the transaction containing `transact_swap`.
4. The transaction fails during Anchor account resolution because `init` is called on a PDA that already exists and has balance/data.
5. The targeted nullifier is permanently bricked from being used in swaps, forcing the user to regenerate a ZK proof with new commitments.

---

## 3. Fix

Use `init_if_needed` for the executor PDA and check whether the executor is already active, resetting its values if needed:

```rust
// In transact_swap:
#[account(
    init_if_needed,
    payer = relayer,
    seeds = [b"executor_v3", source_mint.as_ref(), dest_mint.as_ref(), input_nullifier_0.as_ref()],
    bump,
    space = Executor::LEN
)]
pub executor: Account<'info, Executor>,
```
In the handler, verify that if it is already initialized, it matches the current swap context.
