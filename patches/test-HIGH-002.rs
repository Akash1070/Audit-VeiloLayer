// =========================================================================
// SECURITY TEST — HIGH-002: Position PDA Key Unbound in ZK Proof
// Finding:   findings/HIGH-002-position-pda-key-unbound.md
// Patch:     patches/patch-HIGH-002.rs
// PoC:       poc/poc-HIGH-002.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high002_substituted_position_pda_key_rejected() {
        let claimant = Pubkey::new_unique();
        let withdrawal_id = [7u8; 32];
        
        // Attacker constructs a substitute position PDA key
        let wrong_key = [99u8; 32];

        let result = program.open_position(
            position_pda_key = wrong_key,
            claimant = claimant,
            withdrawal_id = withdrawal_id,
            ...
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PrivacyError::InvalidSwapParams);
    }
}
