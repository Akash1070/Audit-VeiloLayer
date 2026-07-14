# VeiloLayer `privacy_pool` Smart Contract Security Audit Report

This repository contains the comprehensive security audit report, patches, and proof-of-concept (PoC) exploit scripts for the **VeiloLayer `privacy_pool`** Solana smart contract.

**Program ID:** `GYy4kM6GHhpgLCUscuABbzkD2ZbJ2fneYryaZ6Ch7fFU`  
**Repository Audited:** `https://github.com/VeiloSolana/privacy-program`  
**Audit Date:** July 14, 2026  
**Target Scope:** Rust contract, ZK-SNARK verification inputs, PDA derivation, and accounting.

---

## Executive Summary

A comprehensive, low-level security review was conducted on the VeiloLayer `privacy_pool` codebase. The scope of the audit focused strictly on vulnerabilities that could cause direct user fund loss, double-spending, unauthorized pool depletion, or ZK-SNARK verification bypasses.

The audit identified **2 CRITICAL**, **5 HIGH**, **2 MEDIUM**, and **2 LOW** severity vulnerabilities. Crucially, the replay-protection logic in reissue paths can be bypassed, and TVL accounting can be inflated due to inconsistent order of operations.

---

## Vulnerabilities Summary Table

| ID | Severity | Title | Impact | Writeup Link |
|----|----------|-------|--------|--------------|
| **CRIT-001** | 🔴 CRITICAL | `init_if_needed` Nullifier Bypass on Reissue Paths | Proof replay leading to double-minted private notes and vault depletion. | [CRIT-001 writeup](./findings/CRIT-001-nullifier-bypass.md) |
| **CRIT-002** | 🔴 CRITICAL | Relayer-Float Path Skips Vault Debit | TVL accounting inflation resulting in phantom pool capacity. | [CRIT-002 writeup](./findings/CRIT-002-relayer-float-vault-debit.md) |
| **HIGH-001** | 🟠 HIGH | Jupiter Route Injection (No Intermediate Mint Check) | Malicious whitelisted relayer can siphon swap value via fake intermediate tokens. | [HIGH-001 writeup](./findings/HIGH-001-jupiter-route-injection.md) |
| **HIGH-002** | 🟠 HIGH | Position PDA Key Unbound in ZK Proof | Position hijacking via public input substitution in proof verification. | [HIGH-002/HIGH-003 writeup](./findings/HIGH-002-003-MED-001-findings.md#high-002-position-pda-key-not-bound-in-zk-proof---position-hijacking) |
| **HIGH-003** | 🟠 HIGH | Phoenix `ember_unwrap` Over-Credit before Cap Check | Accumulation of reissued USDC beyond the designated withdrawal cap. | [HIGH-002/HIGH-003 writeup](./findings/HIGH-002-003-MED-001-findings.md#high-003-phoenix-ember_unwrap---cumulative-over-credit-before-slot-cap-check) |
| **NEW-001** | 🟠 HIGH | Cross-Namespace Nullifier Reuse | Double-spending nullifiers across `transact_swap` and `transact` instructions. | [NEW-001 writeup](./findings/NEW-001-cross-namespace-nullifier-reuse.md) |
| **NEW-005** | 🟠 HIGH | `fund_native_open_position` Missing Instruction Pairing | Atomic check bypass allowing relayer to drain funded WSOL. | [NEW-005 writeup](./findings/NEW-005-fund-native-position-bypass.md) |
| **MED-001** | 🟡 MEDIUM | `reduce_to_field` Off-by-One on `FR_MODULUS` Equality | Input exactly matching `FR_MODULUS` results in a degenerate zero element. | [HIGH-002/HIGH-003 writeup](./findings/HIGH-002-003-MED-001-findings.md#medium-001-reduce_to_field-off-by-one---fr_modulus-itself-not-reduced) |
| **NEW-004** | 🟡 MEDIUM | Executor PDA Pre-Seeding DoS / Rent-Griefing | Permanent DoS locking users out of swapping specific nullifiers. | [NEW-004 writeup](./findings/NEW-004-executor-pda-dos.md) |
| **NEW-002** | 🟡 MEDIUM | Permissionless Relayer on Deposits | Timing privacy leakage and fee front-running vulnerability. | [NEW-002 writeup](./findings/NEW-002-deposit-relayer-bypass.md) |
| **NEW-003** | 🟢 LOW | `stage_swap_legs` Buffer Linkage DoS | Griefing vector allowing relayers to invalidate user proofs before submission. | [NEW-003 writeup](./findings/NEW-003-stage-swap-legs-dos.md) |
| **NEW-006** | 🟢 LOW | No Admin Rotation Timelock | Key compromise risk without multi-sig or timelock safeguards. | [NEW-006 writeup](./findings/NEW-006-no-timelock.md) |

---

## Detailed Vulnerability Breakdown & Writeups

For full details, including root cause, step-by-step attack vectors, and specific code line references, please navigate to the respective finding documentation:

1. **[CRIT-001: init_if_needed Nullifier Bypass](./findings/CRIT-001-nullifier-bypass.md)**
   * **Vulnerability:** Anchor's `init_if_needed` constraint is used for nullifier markers in reissue paths (`jperp_reissue_notes`, `jperp_recover_native`, `phoenix_reissue_notes`, `prediction_reissue`).
   * **Impact:** Reissue nullifiers can be replayed to double-mint private notes.
2. **[CRIT-002: Relayer-Float Path Skips Vault Debit](./findings/CRIT-002-relayer-float-vault-debit.md)**
   * **Vulnerability:** The native SOL float path bypasses direct vault debit, updating only the TVL configuration counter.
   * **Impact:** Mismatch between on-chain TVL state and real vault lamports, leading to phantom TVL inflation.
3. **[HIGH-001: Jupiter Route Injection](./findings/HIGH-001-jupiter-route-injection.md)**
   * **Vulnerability:** Jupiter standard swap routes have no intermediate pool or token mint validation checks.
   * **Impact:** Attacker LPs can route swap legs through malicious custom pools to extract value.
4. **[HIGH-002 / HIGH-003 / MED-001: ZK Inputs, Phoenix Cap Check, reduce_to_field](./findings/HIGH-002-003-MED-001-findings.md)**
   * **HIGH-002:** `position_pda_key` is not bound to ZK public inputs, enabling relayer hijacking of opened positions.
   * **HIGH-003:** Phoenix `ember_unwrap` cap check runs pre-increment, allowing cumulative over-credit of USDC.
   * **MED-001:** `reduce_to_field` comparison loop has an off-by-one error, letting `FR_MODULUS` equal inputs bypass reduction to `0`.
5. **[NEW-001: Cross-Namespace Nullifier Reuse](./findings/NEW-001-cross-namespace-nullifier-reuse.md)**
   * Double-spending nullifiers across instruction namespaces.
6. **[NEW-002: Deposit Relayer Whitelist Bypass](./findings/NEW-002-deposit-relayer-bypass.md)**
   * Bypassing whitelists on deposit operations causing privacy reduction and fee manipulation.
7. **[NEW-003: stage_swap_legs Buffer Linkage DoS](./findings/NEW-003-stage-swap-legs-dos.md)**
   * Buffer PDA validation race condition allowing DoS.
8. **[NEW-004: Executor PDA Rent-Griefing DoS](./findings/NEW-004-executor-pda-dos.md)**
   * Rent-injection preventing future swaps on targeted nullifiers.
9. **[NEW-005: fund_native_open_position Missing Instruction Pairing](./findings/NEW-005-fund-native-position-bypass.md)**
   * Isolated funding calls enabling unauthorized WSOL extraction from vault.
10. **[NEW-006: No Admin Rotation Timelock](./findings/NEW-006-no-timelock.md)**
    * Instant configuration updates without timelocks.

---

## Patches & Remediation

Concrete rust-level code remediations have been implemented for all vulnerability classes:
*   **[Core Code Fixes](./patches/all-patches.rs):** Provides before/after diffs for all critical and high findings (replacing `init_if_needed` with `init`, adding position key derivations, fixing cap increments, and correct FR_MODULUS reduction).

---

## Security Tests & Proofs of Concept

To verify both the exploits and the corresponding fixes, the following validation suites are provided:
*   **[Rust Security Tests](./patches/security_tests.rs):** Unit/integration tests written for the Anchor test framework to assert invariant correctness under attack conditions.
*   **[TypeScript PoC Suite](./poc/security_poc_suite.ts):** Client-side scripts to construct exploit transactions demonstrating replay attacks, accounting mismatches, and field reductions.

---
