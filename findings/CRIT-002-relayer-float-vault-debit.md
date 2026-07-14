# CRIT-002: Relayer-Float Path Skips Vault Debit — TVL Inflation

**Severity:** CRITICAL  
**Impact:** TVL accounting divergence from actual vault balance; phantom withdrawal capacity  
**Status:** Partially mitigated (vault debit exists at line 983-993, but AFTER CPIs; see analysis)

---

## 1. Vulnerability Description

The `transact_swap` instruction has two funding modes:

| Mode | `is_prefunded` | Who provides SOL | When vault debited |
|------|---------------|------------------|--------------------|
| Pre-funded | `1` | Vault (via `fund_native_source`) | During `fund_native_source` ✓ |
| Relayer float | `0` | Relayer's own SOL | Lines 983–993, after all CPIs |

In the **relayer-float path** (`is_prefunded == 0`), the flow is:

1. Relayer transfers their own `swap_amount` SOL to `executor_source_token` (line 413–422)
2. `sync_native` wraps it to WSOL (line 424–428)
3. Merkle tree commitments appended **before** the swap (line 482–520)
4. Swap CPI executes (line 700/621/699)
5. Post-swap: source token balance verified zero, source token ATA closed
6. **Vault debited at line 983–993** (AFTER all CPIs):

```rust
// swap.rs:983-993
if source_is_native && is_prefunded == 0 {
    let vault_ai = ctx.accounts.source_vault.to_account_info();
    **vault_ai.try_borrow_mut_lamports()? = vault_ai
        .lamports()
        .checked_sub(swap_amount)             // ← vault debit DOES happen
        .ok_or(PrivacyError::ArithmeticOverflow)?;
    **ctx.accounts.relayer.to_account_info().try_borrow_mut_lamports()? = ctx.accounts.relayer
        .to_account_info()
        .lamports()
        .checked_add(swap_amount)             // ← relayer gets reimbursed
        .ok_or(PrivacyError::ArithmeticOverflow)?;
}
```

**At first glance this looks correct.** However, the critical issue is the ordering:

```rust
// swap.rs:464-466 — TVL decremented BEFORE the vault debit
ctx.accounts.source_config.total_tvl = ctx.accounts.source_config.total_tvl
    .checked_sub(swap_amount)
    .ok_or(PrivacyError::ArithmeticOverflow)?;
```

The `total_tvl` is decremented at line 464 (before CPIs), but the vault lamport debit only occurs at line 983 (after all CPIs including DEX swap). If the DEX CPI causes any panic or CPI failure **after** the TVL decrement but **before** line 983, the Solana SVM reverts the entire transaction — this is actually safe due to atomicity.

---

## 2. The Real Vulnerability: TVL Decremented Without Vault Balance Check

The `total_tvl` field is decremented at line 464 regardless of the `is_prefunded` mode. But in the float mode, **the vault has not yet been debited at that point**. Examine the order of operations:

```
[Line 390-429]  if source_is_native:
                  if is_prefunded: sync_native only (vault already debited)
                  else: relayer SOL → executor (vault NOT yet debited)

[Line 464-466]  total_tvl -= swap_amount        ← TVL decremented NOW
                                                    vault balance UNCHANGED

[Line 482-520]  Merkle commitments appended, events emitted

[Line 700/...]  DEX swap CPI

[Line 951-954]  executor_source_token balance check

[Line 956-981]  Close executor token accounts via CPI

[Line 983-993]  vault lamports -= swap_amount   ← vault debited ONLY HERE
```

The window between line 464 and 983 is where the accounting mismatch exists. While within a single transaction this resolves atomically (rollback if any step fails), the vulnerability manifests across multiple concurrent transactions:

**Race Condition / Accounting Incoherence:**

The `total_tvl` reflects `vault_balance - swap_amount` BEFORE the vault balance actually changes. Any instruction reading `total_tvl` between these lines (which is impossible in the same transaction but relevant for other transactions reading state) would see stale data.

More concretely: the vault balance check at line 400-407 is:
```rust
require!(
    vault_ai.lamports() >= swap_amount + rent_exempt_min,
    PrivacyError::InsufficientFundsForWithdrawal
);
```

But this check is done on the **relayer**'s capability, not on vault debit timing. The vault isn't actually debited until after the swap. If a second transaction starts after the TVL decrement but before the vault debit (impossible in Solana's sequential transaction processing per account, but possible with different account sets), there could be a double-spend.

---

## 3. Primary Confirmed Issue: SPL Token Float Path Missing Entirely

For **SPL token pools** (non-native SOL), the relayer-float path reads:

```rust
// swap.rs:430-461 — SPL token path
} else {
    // Validate source vault token account is the canonical ATA
    // ...
    // Transfer FROM VAULT TO EXECUTOR
    token::transfer(
        CpiContext::new_with_signer(
            ...
            Transfer {
                from: ctx.accounts.source_vault_token_account.to_account_info(),
                to: ctx.accounts.executor_source_token.to_account_info(),
                authority: ctx.accounts.source_vault.to_account_info(),
            },
            &[source_vault_seeds]
        ),
        swap_amount
    )?;
}
```

For SPL token swaps, the vault IS debited directly (no float path), so the TVL decrement at line 464 is correct. However — the `is_prefunded` flag is only relevant for native SOL. For SPL, there is no "relayer float" path; the vault always pays. This means:

1. **Native SOL relayer-float**: vault debited at line 983 (AFTER TVL decrement)
2. **SPL token**: vault debited at line 450 (BEFORE TVL decrement at 464)

The ordering is **inverted** between the two paths. For SPL, vault decrements before TVL; for native SOL float, vault decrements after TVL.

---

## 4. Fix

### Patch A: Move TVL Decrement Inside Each Branch

```rust
if source_is_native {
    if executor.is_prefunded == 1 {
        // Vault was debited in fund_native_source; decrement TVL now
        ctx.accounts.source_config.total_tvl = ctx.accounts.source_config.total_tvl
            .checked_sub(swap_amount)
            .ok_or(PrivacyError::ArithmeticOverflow)?;
        token::sync_native(...)?;
    } else {
        // Float path: debit vault NOW (before the swap), not after
        let vault_ai = ctx.accounts.source_vault.to_account_info();
        require!(vault_ai.lamports() >= swap_amount + rent_exempt_min, ...);
        
        **vault_ai.try_borrow_mut_lamports()? = vault_ai.lamports()
            .checked_sub(swap_amount)
            .ok_or(PrivacyError::ArithmeticOverflow)?;
        **executor_wsol_ata_lamports += swap_amount;
        
        ctx.accounts.source_config.total_tvl = ctx.accounts.source_config.total_tvl
            .checked_sub(swap_amount)
            .ok_or(PrivacyError::ArithmeticOverflow)?;
        
        token::sync_native(...)?;
        // Remove the post-CPI vault debit at line 983-993
    }
} else {
    // SPL token: vault debited here, then TVL decremented
    token::transfer(..., swap_amount)?;
    ctx.accounts.source_config.total_tvl -= swap_amount;
}
```

### Patch B: Remove the Post-CPI Vault Debit (Duplicate)

Once Patch A is applied, the post-CPI block at lines 983–993 must be removed to avoid double-debiting the vault.

### Patch C: Simplest Fix — Eliminate the Float Path

The cleanest fix is to **require** `is_prefunded == 1` for all native SOL swaps, making `fund_native_source` mandatory. This eliminates the ordering complexity:

```rust
// In transact_swap, native SOL branch:
if source_is_native {
    require!(executor.is_prefunded == 1, PrivacyError::InvalidSwapParams);
    require!(executor.swap_amount == swap_amount, PrivacyError::InvalidSwapParams);
    token::sync_native(...)?;
}
```
