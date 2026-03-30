//! ELF64 relocation parsing (x86-64 and AArch64).

use crate::parser::{ElfError, read_u64_le};

// x86-64 relocation types
pub const R_X86_64_NONE: u32 = 0;
pub const R_X86_64_64: u32 = 1;
pub const R_X86_64_PC32: u32 = 2;
pub const R_X86_64_32: u32 = 10;
pub const R_X86_64_32S: u32 = 11;
pub const R_X86_64_RELATIVE: u32 = 8;

// AArch64 relocation types
pub const R_AARCH64_NONE: u32 = 0;
pub const R_AARCH64_ABS64: u32 = 257;
pub const R_AARCH64_PREL32: u32 = 261;
pub const R_AARCH64_RELATIVE: u32 = 1027;

/// Parsed ELF64 RELA entry (with explicit addend).
#[derive(Debug, Clone, Copy)]
pub struct Elf64Rela {
    pub offset: u64,
    pub info: u64,
    pub addend: i64,
}

impl Elf64Rela {
    /// Symbol index from r_info.
    pub fn sym_index(&self) -> u32 {
        (self.info >> 32) as u32
    }

    /// Relocation type from r_info.
    pub fn rel_type(&self) -> u32 {
        self.info as u32
    }
}

/// Parse a single RELA entry at the given offset.
pub fn parse_elf64_rela(data: &[u8], offset: usize) -> Result<Elf64Rela, ElfError> {
    if offset + 24 > data.len() {
        return Err(ElfError::SectionOutOfBounds);
    }

    Ok(Elf64Rela {
        offset: read_u64_le(data, offset),
        info: read_u64_le(data, offset + 8),
        addend: read_u64_le(data, offset + 16) as i64,
    })
}

/// Iterator over RELA entries in a relocation section.
pub struct RelaIter<'a> {
    data: &'a [u8],
    offset: usize,
    end: usize,
}

impl<'a> RelaIter<'a> {
    pub fn new(rela_data: &'a [u8]) -> Self {
        Self {
            data: rela_data,
            offset: 0,
            end: rela_data.len(),
        }
    }
}

impl<'a> Iterator for RelaIter<'a> {
    type Item = Elf64Rela;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + 24 > self.end {
            return None;
        }
        let rela = parse_elf64_rela(self.data, self.offset).ok()?;
        self.offset += 24;
        Some(rela)
    }
}

/// Apply a single x86-64 relocation.
///
/// `target` is the mutable memory where the relocation is applied.
/// `sym_value` is the resolved symbol address.
/// `base_addr` is the load base address (for RELATIVE relocations).
///
/// Returns `Ok(())` on success, or `Err(ElfError)` if the relocation
/// type is unsupported or the target offset is out of bounds.
pub fn apply_x86_64_rela(
    target: &mut [u8],
    rela: &Elf64Rela,
    sym_value: u64,
    base_addr: u64,
) -> Result<(), ElfError> {
    let off = rela.offset as usize;
    let s = sym_value;
    let a = rela.addend;

    match rela.rel_type() {
        R_X86_64_NONE => {}
        R_X86_64_64 => {
            let val = (s as i64).wrapping_add(a) as u64;
            if off + 8 > target.len() {
                return Err(ElfError::SectionOutOfBounds);
            }
            target[off..off + 8].copy_from_slice(&val.to_le_bytes());
        }
        R_X86_64_PC32 => {
            let val = (s as i64).wrapping_add(a).wrapping_sub((base_addr + rela.offset) as i64) as u32;
            if off + 4 > target.len() {
                return Err(ElfError::SectionOutOfBounds);
            }
            target[off..off + 4].copy_from_slice(&val.to_le_bytes());
        }
        R_X86_64_32 => {
            let val = (s as i64).wrapping_add(a) as u32;
            if off + 4 > target.len() {
                return Err(ElfError::SectionOutOfBounds);
            }
            target[off..off + 4].copy_from_slice(&val.to_le_bytes());
        }
        R_X86_64_32S => {
            let val = (s as i64).wrapping_add(a) as i32;
            if off + 4 > target.len() {
                return Err(ElfError::SectionOutOfBounds);
            }
            target[off..off + 4].copy_from_slice(&val.to_le_bytes());
        }
        R_X86_64_RELATIVE => {
            let val = (base_addr as i64).wrapping_add(a) as u64;
            if off + 8 > target.len() {
                return Err(ElfError::SectionOutOfBounds);
            }
            target[off..off + 8].copy_from_slice(&val.to_le_bytes());
        }
        _ => return Err(ElfError::SectionOutOfBounds), // Unsupported type
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rela(offset: u64, sym: u32, rtype: u32, addend: i64) -> [u8; 24] {
        let mut buf = [0u8; 24];
        buf[0..8].copy_from_slice(&offset.to_le_bytes());
        let info = ((sym as u64) << 32) | (rtype as u64);
        buf[8..16].copy_from_slice(&info.to_le_bytes());
        buf[16..24].copy_from_slice(&addend.to_le_bytes());
        buf
    }

    #[test]
    fn test_parse_rela() {
        let data = make_rela(0x1000, 5, R_X86_64_64, -8);
        let rela = parse_elf64_rela(&data, 0).unwrap();
        assert_eq!(rela.offset, 0x1000);
        assert_eq!(rela.sym_index(), 5);
        assert_eq!(rela.rel_type(), R_X86_64_64);
        assert_eq!(rela.addend, -8);
    }

    #[test]
    fn test_rela_iter() {
        let mut data = [0u8; 48];
        data[0..24].copy_from_slice(&make_rela(0x100, 1, R_X86_64_64, 0));
        data[24..48].copy_from_slice(&make_rela(0x200, 2, R_X86_64_PC32, 4));

        let entries: Vec<_> = RelaIter::new(&data).collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].offset, 0x100);
        assert_eq!(entries[1].rel_type(), R_X86_64_PC32);
    }

    #[test]
    fn test_apply_r_x86_64_64() {
        let mut target = [0u8; 16];
        let rela = Elf64Rela { offset: 0, info: ((0u64) << 32) | R_X86_64_64 as u64, addend: 8 };
        apply_x86_64_rela(&mut target, &rela, 0x1000, 0).unwrap();
        let val = u64::from_le_bytes(target[0..8].try_into().unwrap());
        assert_eq!(val, 0x1008);
    }

    #[test]
    fn test_apply_r_x86_64_relative() {
        let mut target = [0u8; 16];
        let rela = Elf64Rela { offset: 0, info: R_X86_64_RELATIVE as u64, addend: 0x500 };
        apply_x86_64_rela(&mut target, &rela, 0, 0x200000).unwrap();
        let val = u64::from_le_bytes(target[0..8].try_into().unwrap());
        assert_eq!(val, 0x200500);
    }

    #[test]
    fn test_apply_none() {
        let mut target = [0xFFu8; 8];
        let rela = Elf64Rela { offset: 0, info: R_X86_64_NONE as u64, addend: 0 };
        apply_x86_64_rela(&mut target, &rela, 0, 0).unwrap();
        assert_eq!(target, [0xFF; 8]); // Unchanged
    }
}

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
use alloc::vec::Vec;
