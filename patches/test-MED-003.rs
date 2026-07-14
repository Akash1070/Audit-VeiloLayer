// =========================================================================
// SECURITY TEST — MED-003: Executor PDA Rent-Griefing DoS
// Finding:   findings/MED-003-executor-pda-dos.md
// Patch:     patches/patch-MED-003.rs
// PoC:       poc/poc-MED-003.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_med003_rent_prefunded_executor_resolves_correctly() {
        let nullifier = [8u8; 32];
        let relayer = Keypair::new();

        // 1. Attacker pre-funds the deterministic executor PDA with rent-exempt lamports
        let executor_pda = find_executor_pda(relayer.pubkey(), nullifier);
        send_lamports(executor_pda, 2_000_000); // 0.002 SOL

        // 2. Relayer submits swap.
        // Expect: Patched code uses init_if_needed, so transaction succeeds instead of failing.
        let result = program.transact_swap(relayer = relayer, nullifier = nullifier, ...);
        assert!(result.is_ok());
    }
}
