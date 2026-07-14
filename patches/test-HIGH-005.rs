// =========================================================================
// SECURITY TEST — HIGH-005: fund_native_open_position Missing Instruction Pairing
// Finding:   findings/HIGH-005-fund-native-position-bypass.md
// Patch:     patches/patch-HIGH-005.rs
// PoC:       poc/poc-HIGH-005.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high005_isolated_funding_rejected() {
        // Attempt to call fund_native_open_position as a standalone transaction
        let result = program.fund_native_open_position(amount = 10_000_000_000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PrivacyError::MissingOpenPositionInstruction);
    }
}
