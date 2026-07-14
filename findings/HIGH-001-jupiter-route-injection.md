# HIGH-001: Jupiter Route Injection — No Intermediate Mint Validation

**Severity:** HIGH  
**Impact:** Attacker LP captures value via intermediate swap hops  

## Root Cause

`SharedAccountsRoute` validates source/dest mints at `remaining[7]` and `remaining[8]`. The standard `Route` / `ExactOutRoute` path (swap.rs:784–822) passes all `remaining[4+]` accounts through unchanged with **zero mint or pool validation**.

## Attack Path

A malicious whitelisted relayer constructs a multi-leg Jupiter route where one leg routes through an attacker-controlled pool at an unfavorable rate. The `min_amount_out` check provides partial protection but can be bypassed by setting slippage tolerance to the extraction amount.

The `swap_data_hash` check (line 827–828) does bind the exact bytes — but only if the user trusts the route the relayer showed them at proof time. A relayer can show a valid route, get the ZK proof, then present the same route bytes (passing the hash check) where one of the pool accounts is replaced with an attacker account at the same address (impossible — but route legs pointing to legitimate-looking fake pools are feasible).

## Fix

```rust
// Add after minimum account check in standard Route branch:
// Verify executor token account mints match instruction args
require!(
    ctx.accounts.executor_dest_token.mint == effective_mint(&dest_mint),
    PrivacyError::InvalidMintAddress
);
// Also enforce swap_data_hash non-zero for all Jupiter routes
require!(
    swap_params.swap_data_hash != [0u8; 32],
    PrivacyError::JupiterInvalidInstruction
);
```
