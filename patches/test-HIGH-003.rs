// =========================================================================
// SECURITY TEST — HIGH-003: Phoenix ember_unwrap Cumulative Over-Credit
// Finding:   findings/HIGH-003-phoenix-unwrap-overcredit.md
// Patch:     patches/patch-HIGH-003.rs
// PoC:       poc/poc-HIGH-003.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high003_phoenix_unwrap_exceeds_cap_fails() {
        // Set slot cap at 100 USDC
        let slot_cap = 100_000_000;

        // First unwrap (60 USDC) - succeeds
        let res1 = program.phoenix_ember_unwrap(amount = 60_000_000);
        assert!(res1.is_ok());

        // Second unwrap (60 USDC) - should fail because cumulative exceeds cap (120 > 100)
        let res2 = program.phoenix_ember_unwrap(amount = 60_000_000);
        assert!(res2.is_err());
        assert_eq!(res2.unwrap_err(), PrivacyError::SlotOverdraft);
    }
}
