# VeiloLayer `privacy_pool` Smart Contract Security Audit Report

This repository contains the comprehensive security audit report, patches, and proof-of-concept (PoC) exploit scripts for the **VeiloLayer `privacy_pool`** Solana smart contract.

**Program ID:** `GYy4kM6GHhpgLCUscuABbzkD2ZbJ2fneYryaZ6Ch7fFU`  
**Repository Audited:** `https://github.com/VeiloSolana/privacy-program`  
**Audit Date:** July 14, 2026  
**Target Scope:** Rust contract, ZK-SNARK verification inputs, PDA derivation, and accounting.

---

## Executive Summary

A comprehensive, low-level security review was conducted on the VeiloLayer `privacy_pool` codebase. The scope of the audit focused strictly on vulnerabilities that could cause direct user fund loss, double-spending, unauthorized pool depletion, or ZK-SNARK verification bypasses.

The audit identified **2 CRITICAL**, **5 HIGH**, **3 MEDIUM**, and **2 LOW** severity vulnerabilities. Crucially, the replay-protection logic in reissue paths can be bypassed, and TVL accounting can be inflated due to inconsistent order of operations.

---

### Vulnerabilities Summary Table

| ID | Severity | Title | Impact | Writeup Link |
|----|----------|-------|--------|--------------|
| **CRIT-001** | 🔴 CRITICAL | `init_if_needed` Nullifier Bypass on Reissue Paths | Proof replay leading to double-minted private notes and vault depletion. | [CRIT-001 writeup](./findings/CRIT-001-nullifier-bypass.md) |
| **CRIT-002** | 🔴 CRITICAL | Relayer-Float Path Skips Vault Debit | TVL accounting inflation resulting in phantom pool capacity. | [CRIT-002 writeup](./findings/CRIT-002-relayer-float-vault-debit.md) |
| **HIGH-001** | 🟠 HIGH | Jupiter Route Injection (No Intermediate Mint Check) | Malicious whitelisted relayer can siphon swap value via fake intermediate tokens. | [HIGH-001 writeup](./findings/HIGH-001-jupiter-route-injection.md) |
| **HIGH-002** | 🟠 HIGH | Position PDA Key Unbound in ZK Proof | Position hijacking via public input substitution in proof verification. | [HIGH-002 writeup](./findings/HIGH-002-position-pda-key-unbound.md) |
| **HIGH-003** | 🟠 HIGH | Phoenix `ember_unwrap` Over-Credit before Cap Check | Accumulation of reissued USDC beyond the designated withdrawal cap. | [HIGH-003 writeup](./findings/HIGH-003-phoenix-unwrap-overcredit.md) |
| **HIGH-004** | 🟠 HIGH | Cross-Namespace Nullifier Reuse | Double-spending nullifiers across `transact_swap` and `transact` instructions. | [HIGH-004 writeup](./findings/HIGH-004-cross-namespace-nullifier-reuse.md) |
| **HIGH-005** | 🟠 HIGH | `fund_native_open_position` Missing Instruction Pairing | Atomic check bypass allowing relayer to drain funded WSOL. | [HIGH-005 writeup](./findings/HIGH-005-fund-native-position-bypass.md) |
| **MED-001** | 🟡 MEDIUM | `reduce_to_field` Off-by-One on `FR_MODULUS` Equality | Input exactly matching `FR_MODULUS` results in a degenerate zero element. | [MED-001 writeup](./findings/MED-001-reduce-to-field-off-by-one.md) |
| **MED-002** | 🟡 MEDIUM | Deposit Relayer Whitelist Bypass | Timing privacy leakage and fee front-running vulnerability. | [MED-002 writeup](./findings/MED-002-deposit-relayer-bypass.md) |
| **MED-003** | 🟡 MEDIUM | Executor PDA Pre-Seeding DoS / Rent-Griefing | Permanent DoS locking users out of swapping specific nullifiers. | [MED-003 writeup](./findings/MED-003-executor-pda-dos.md) |
| **LOW-001** | 🟢 LOW | `stage_swap_legs` Buffer Linkage DoS | Griefing vector allowing relayers to invalidate user ZK proofs. | [LOW-001 writeup](./findings/LOW-001-stage-swap-legs-dos.md) |
| **LOW-002** | 🟢 LOW | No Admin Rotation Timelock | Key compromise risk without multi-sig or timelock safeguards. | [LOW-002 writeup](./findings/LOW-002-no-timelock.md) |

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
4. **[HIGH-002: Position PDA Key Unbound in ZK Proof](./findings/HIGH-002-position-pda-key-unbound.md)**
   * **Vulnerability:** `position_pda_key` is not bound to ZK public inputs, enabling relayer hijacking of opened positions.
5. **[HIGH-003: Phoenix ember_unwrap Cap Check Bypass](./findings/HIGH-003-phoenix-unwrap-overcredit.md)**
   * **Vulnerability:** Phoenix `ember_unwrap` cap check runs pre-increment, allowing cumulative over-credit of USDC.
6. **[MED-001: reduce_to_field Off-by-One on FR_MODULUS Equality](./findings/MED-001-reduce-to-field-off-by-one.md)**
   * **Vulnerability:** `reduce_to_field` comparison loop has an off-by-one error, letting `FR_MODULUS` equal inputs bypass reduction to `0`.
7. **[HIGH-004: Cross-Namespace Nullifier Reuse](./findings/HIGH-004-cross-namespace-nullifier-reuse.md)**
   * **Vulnerability:** Nullifiers spent in `transact_swap` use the `source_nullifier_v3` prefix and do not prevent the same nullifier from being spent in the `transact` namespace.
   * **Impact:** Double-spending nullifiers across instruction namespaces.
8. **[HIGH-005: fund_native_open_position Missing Instruction Pairing](./findings/HIGH-005-fund-native-position-bypass.md)**
   * **Vulnerability:** `fund_native_open_position` lacks sysvar instruction pairing checks, permitting relayers to fund the executor in isolation.
   * **Impact:** Isolated funding calls enabling unauthorized WSOL extraction from vault.
9. **[MED-002: Deposit Relayer Whitelist Bypass](./findings/MED-002-deposit-relayer-bypass.md)**
   * **Vulnerability:** Deposit operations bypass the relayer whitelist check, allowing non-whitelisted relayers to intercept deposit transactions.
   * **Impact:** Privacy reduction, fee manipulation, and rent griefing.
10. **[MED-003: Executor PDA Rent-Griefing DoS](./findings/MED-003-executor-pda-dos.md)**
    * **Vulnerability:** Executor PDA uses `init` instead of `init_if_needed` and leaves the account open on transaction completion.
    * **Impact:** Rent-injection preventing future swaps on targeted nullifiers.
11. **[LOW-001: stage_swap_legs Buffer Linkage DoS](./findings/LOW-001-stage-swap-legs-dos.md)**
    * **Vulnerability:** Staged swap route buffer PDAs can be deleted or closed by relayers before execution.
    * **Impact:** Griefing vector allowing relayers to invalidate user ZK proofs.
12. **[LOW-002: No Admin Rotation Timelock](./findings/LOW-002-no-timelock.md)**
    * **Vulnerability:** Pool global configuration parameters can be updated instantly by the admin authority.
    * **Impact:** Catastrophic single point of failure and lack of grace period under admin key compromise.

---

## Patches & Remediation

Concrete rust-level code remediations have been implemented for all vulnerability classes:
*   **[Remediation Patches](./patches/):** Provides individual `patch-*.rs` before/after fix diffs for all findings.

---

## Security Tests & Proofs of Concept

To verify both the exploits and the corresponding fixes, the following validation suites are provided:
*   **[Rust Security Tests](./patches/):** Individual `test-*.rs` unit/integration tests written for the Anchor test framework to assert invariant correctness under attack conditions.
*   **[TypeScript PoCs](./poc/):** Individual `poc-*.ts` client-side scripts to construct exploit transactions demonstrating replay attacks, accounting mismatches, and field reductions.

---
