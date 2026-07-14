// =========================================================================
// SECURITY TEST — LOW-002: No Admin Rotation Timelock
// Finding:   findings/LOW-002-no-timelock.md
// Patch:     patches/patch-LOW-002.rs
// PoC:       poc/poc-LOW-002.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low002_admin_rotation_enforces_timelock() {
        let admin = Keypair::new();
        let new_admin = Keypair::new();

        // 1. Propose new admin
        program.propose_admin(admin = admin, new_admin = new_admin.pubkey()).unwrap();

        // 2. Attempt to claim immediately (should fail before timelock expires)
        let claim_fail = program.claim_admin(new_admin = new_admin);
        assert!(claim_fail.is_err());
        assert_eq!(claim_fail.unwrap_err(), PrivacyError::TimelockNotExpired);

        // 3. Fast-forward clock and claim (should succeed)
        fast_forward_time(259200); // 3 days
        let claim_ok = program.claim_admin(new_admin = new_admin);
        assert!(claim_ok.is_ok());
    }
}
