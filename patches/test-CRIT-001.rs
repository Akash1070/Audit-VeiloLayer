// =========================================================================
// SECURITY TEST — CRIT-001: Nullifier Replay on Reissue Paths
// Finding:   findings/CRIT-001-nullifier-bypass.md
// Patch:     patches/patch-CRIT-001.rs
// PoC:       poc/poc-CRIT-001.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;
    use anchor_lang::InstructionData;

    #[test]
    fn test_crit001_nullifier_replay_fails_on_constraint() {
        // Setup: Init first reissue path nullifiers N1, N2
        let nullifier_0 = [1u8; 32];
        let nullifier_1 = [2u8; 32];

        // Call 1: First reissue notes with N1, N2.
        // Expect: Success, marker PDA created.
        let res1 = program.jperp_reissue_notes(nullifier_0, nullifier_1, ...);
        assert!(res1.is_ok());

        // Call 2: Attempt duplicate call with N1, N2.
        // Expect: Failure at the Anchor account resolution stage (Constraint init).
        let res2 = program.jperp_reissue_notes(nullifier_0, nullifier_1, ...);
        assert!(res2.is_err());
        let err = res2.unwrap_err();
        
        // Assert that the error is AccountAlreadyInitialized (3012 / 0xbc4), NOT a handler-level error.
        assert!(err.to_string().contains("already in use") || err.to_string().contains("0xbc4"));
    }
}
