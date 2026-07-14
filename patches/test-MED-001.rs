// =========================================================================
// SECURITY TEST — MED-001: reduce_to_field Off-By-One on FR_MODULUS Equality
// Finding:   findings/MED-001-reduce-to-field-off-by-one.md
// Patch:     patches/patch-MED-001.rs
// PoC:       poc/poc-MED-001.ts
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_med001_reduce_to_field_reduces_modulus_to_zero() {
        const FR_MODULUS: [u8; 32] = [
            0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81,
            0x58, 0x5d, 0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91, 0x43, 0xe1, 0xf5, 0x93,
            0xf0, 0x00, 0x00, 0x01,
        ];

        // Patched reduce_to_field must output [0; 32] when input is FR_MODULUS.
        let output = SwapParams::reduce_to_field(FR_MODULUS);
        assert_eq!(output, [0u8; 32], "FR_MODULUS should reduce to 0");
    }
}
