use heck::ToSnakeCase;
use solana_idl::IdlInstruction;
use solana_sdk::hash;

// Namespace for calculating instruction sighash signatures for any instruction
// not affecting program state.
const SIGHASH_GLOBAL_NAMESPACE: &str = "global";

pub fn discriminator_from_ix(ix: &IdlInstruction) -> Vec<u8> {
    ix.discriminant
        .as_ref()
        // Newer Anchor Versions >=0.30 add the discriminator value which
        // is moved to the `bytes` property
        // Shank adds the indes of the instruction to the `value` property
        // instead.
        .map(|x| x.bytes.clone().unwrap_or(vec![x.value]))
        // If we don't find it in either we assume it is an older anchor IDL
        // and derive the discriminator the same way that anchor did before.
        .unwrap_or_else(|| anchor_sighash(SIGHASH_GLOBAL_NAMESPACE, &ix.name).to_vec())
}

/// Replicates the mechanism that anchor used in order to derive a discriminator
/// from the name of an instruction.
fn anchor_sighash(namespace: &str, ix_name: &str) -> [u8; 8] {
    // NOTE: even though the name of the ix is lower camel cased in the IDL it
    // seems that the IX discriminator is derived from the snake case version
    // (see discriminator_for_house_initialize test below which came from a real case)
    let ix_name = ix_name.to_snake_case();

    let preimage = format!("{namespace}:{ix_name}");

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(&hash::hash(preimage.as_bytes()).to_bytes()[..8]);
    sighash
}
