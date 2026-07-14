// =========================================================================
// SECURITY TEST — HIGH-001: Jupiter Route Injection
// Finding:   findings/HIGH-001-jupiter-route-injection.md
// Patch:     patches/patch-HIGH-001.rs
// PoC:       poc/poc-HIGH-001.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high001_jupiter_injected_route_rejected() {
        // Prepare swap params with zero swap_data_hash (malicious bypass)
        let swap_params = SwapParams {
            min_amount_out: 1000,
            deadline: 12345678,
            dest_amount: 1000,
            swap_data_hash: [0u8; 32], // Zero hash injected
        };

        let result = program.transact_swap(swap_params, ...);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PrivacyError::JupiterInvalidInstruction);
    }
}
