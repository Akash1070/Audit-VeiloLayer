// =========================================================================
// SECURITY TEST — CRIT-002: Relayer-Float Path Skips Vault Debit
// Finding:   findings/CRIT-002-relayer-float-vault-debit.md
// Patch:     patches/patch-CRIT-002.rs
// PoC:       poc/poc-CRIT-002.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crit002_relayer_float_is_rejected() {
        // If Option A is applied (disallow float path):
        // Expect transact_swap with is_prefunded == 0 to fail.
        let result = program.transact_swap(is_prefunded = 0, swap_amount = 100_000_000, ...);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PrivacyError::InvalidSwapParams);
    }
}
