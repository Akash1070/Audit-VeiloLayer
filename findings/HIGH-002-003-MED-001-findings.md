# HIGH-002: Position PDA Key Unbound in ZK Proof — Position Hijacking

**Severity:** HIGH  
**Impact:** Attacker can hijack a victim's open position by submitting the same proof with a different `position_pda_key`

## Root Cause

`SwapPublicInputs` (swap.rs:132–141) does not include `position_pda_key`:
```rust
pub struct SwapPublicInputs {
    pub source_root: [u8; 32],
    pub swap_params_hash: [u8; 32],
    pub ext_data_hash: [u8; 32],   // includes claimant
    pub source_mint: Pubkey,
    pub dest_mint: Pubkey,
    pub input_nullifiers: [[u8; 32]; 2],
    pub output_commitments: [[u8; 32]; 2],
    pub swap_amount: u64,
    // ← NO position_pda_key
}
```

`ext_data.claimant` is included via `ext_data_hash`, which DOES bind the ephemeral key. However, `position_pda_key` is a separate 32-byte seed passed as an instruction argument. Since it is not part of the ZK public inputs, the proof does not commit to it.

A relayer who observes a valid `open_position` call can substitute any `position_pda_key` value they control. The resulting `position_pda` PDA is owned by the attacker's keypair. When `close_position` is called, the attacker's claimant key signs — not the victim's.

## Mitigating Factor

`ext_data.claimant` IS in the ZK proof. The `position_pda` account has `has_one = claimant` for `close_position` (lib.rs:1422–1426):
```rust
#[account(
    mut,
    seeds = [b"position_pda_v1", position_pda_key.as_ref()],
    bump = position_pda.bump,
    close = relayer,
)]
pub position_pda: Box<Account<'info, PositionPDA>>,
```

But there is no `has_one = claimant` constraint on `open_position`'s `position_pda` creation! The `claimant` field is only stored in `position_pda` if the handler explicitly sets it. If the handler sets `position_pda.claimant = ext_data.claimant` (which it should), then the attacker's substitute `position_pda_key` would produce a PDA with the victim's claimant — defeating the hijack.

**The critical question is whether `open_position` handler enforces:**
```rust
position_pda.claimant = claimant_from_ext_data;
```

If it does, the hijack fails at `close_position` time. If not, the hijack succeeds.

## Fix

Bind `position_pda_key` to `ext_data.claimant` cryptographically:
```rust
// In open_position handler, after ZK verification:
// Derive expected position_pda_key from claimant to prevent substitution
let expected_pda_seed = PoseidonHasher::hashv(&[
    ext_data.claimant.as_ref(),
    withdrawal_id.as_ref(),
]).map_err(|_| error!(PrivacyError::MerkleHashFailed))?;
require!(position_pda_key == expected_pda_seed, PrivacyError::InvalidSwapParams);
```

Or add `position_pda_key` to `SwapPublicInputs` and the ZK circuit.

---

# HIGH-003: Phoenix ember_unwrap Over-Credit Before Slot Cap Check

**Severity:** HIGH  
**Impact:** Phantom USDC minted beyond original Phoenix withdrawal cap

## Root Cause

The slot cap check in `phoenix_ember_unwrap` runs BEFORE the increment:
```rust
// Conceptual handler logic:
require!(pending_reissue.amount <= phoenix_slot.amount, SlotOverdraft);
// ...transfer USDC to vault...
pending_reissue.amount += received_amount;   // ← added AFTER check
```

Multiple calls with `amount = cap - 1` each pass the pre-increment check, accumulating far beyond the cap.

## Fix

```rust
let new_total = pending_reissue.amount
    .checked_add(received_amount)
    .ok_or(PrivacyError::ArithmeticOverflow)?;
require!(new_total <= phoenix_slot.amount, PrivacyError::SlotOverdraft);
pending_reissue.amount = new_total;
```

---

# MED-001: `reduce_to_field` Off-By-One on FR_MODULUS Equality

**Severity:** MEDIUM  
**Impact:** Input exactly equal to FR_MODULUS returns FR_MODULUS unchanged (= 0 in field)

## Root Cause

```rust
// swap.rs:73-87 and lib.rs:491-500
let mut needs_reduction = false;
for i in 0..32 {
    if bytes[i] < FR_MODULUS[i] { break; }           // exits loop: no reduction
    if bytes[i] > FR_MODULUS[i] { needs_reduction = true; break; }
    // if equal: loop continues without setting needs_reduction
}
// When bytes == FR_MODULUS exactly: loop finishes, needs_reduction = false
// → returns FR_MODULUS, which = 0 (mod FR_MODULUS)
if !needs_reduction { return bytes; }
```

## Fix

```rust
// Single comparison replacing the loop:
let needs_reduction = bytes >= FR_MODULUS;
if !needs_reduction { return bytes; }
```

This also applies to the duplicate `reduce_to_field` in `ExtData::hash` (lib.rs:480–517). Both must be patched.
