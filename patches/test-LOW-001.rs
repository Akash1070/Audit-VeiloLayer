// =========================================================================
// SECURITY TEST — LOW-001: stage_swap_legs Buffer Linkage DoS
// Finding:   findings/LOW-001-stage-swap-legs-dos.md
// Patch:     patches/patch-LOW-001.rs
// PoC:       poc/poc-LOW-001.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low001_unauthorized_close_swap_legs_rejected() {
        let claimant = Keypair::new();
        let malicious_signer = Keypair::new();

        // Expect close_swap_legs to fail if signed by someone other than the claimant
        let result = program.close_swap_legs(
            signer = malicious_signer.pubkey(),
            claimant = claimant.pubkey(),
            ...
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PrivacyError::UnauthorizedSigner);
    }
}
