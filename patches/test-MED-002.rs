// =========================================================================
// SECURITY TEST — MED-002: Deposit Relayer Whitelist Bypass
// Finding:   findings/MED-002-deposit-relayer-bypass.md
// Patch:     patches/patch-MED-002.rs
// PoC:       poc/poc-MED-002.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_med002_non_whitelisted_relayer_rejected_on_deposit() {
        let non_whitelisted_relayer = Keypair::new();

        // Expect transaction to fail when a non-whitelisted relayer tries to submit a deposit
        let result = program.deposit(relayer = non_whitelisted_relayer.pubkey(), public_amount = 100_000_000, ...);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PrivacyError::RelayerNotAllowed);
    }
}
