// =========================================================================
// SECURITY TEST — HIGH-004: Cross-Namespace Nullifier Reuse
// Finding:   findings/HIGH-004-cross-namespace-nullifier-reuse.md
// Patch:     patches/patch-HIGH-004.rs
// PoC:       poc/poc-HIGH-004.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high004_cross_namespace_replay_fails() {
        let nullifier = [5u8; 32];

        // Call 1: transact_swap spends nullifier.
        let res1 = program.transact_swap(nullifier, ...);
        assert!(res1.is_ok());

        // Call 2: transact attempts to spend same nullifier.
        // Expect: Failure because namespaces are consolidated to nullifier_v4.
        let res2 = program.transact(nullifier, ...);
        assert!(res2.is_err());
    }
}
