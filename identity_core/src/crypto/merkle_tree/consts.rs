use core::mem;

// https://en.wikipedia.org/wiki/Merkle_tree#Second_preimage_attack
pub const PREFIX_L: &[u8] = &[0x00];
pub const PREFIX_B: &[u8] = &[0x01];

pub const SIZE_USIZE: usize = mem::size_of::<usize>();
pub const SIZE_U16: usize = mem::size_of::<u16>();
