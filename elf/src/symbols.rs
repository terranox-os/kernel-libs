//! ELF64 symbol table parsing.

use crate::parser::{read_u16_le, read_u32_le, read_u64_le, strtab_lookup, ElfError};

// Symbol binding (upper 4 bits of st_info)
pub const STB_LOCAL: u8 = 0;
pub const STB_GLOBAL: u8 = 1;
pub const STB_WEAK: u8 = 2;

// Symbol type (lower 4 bits of st_info)
pub const STT_NOTYPE: u8 = 0;
pub const STT_OBJECT: u8 = 1;
pub const STT_FUNC: u8 = 2;
pub const STT_SECTION: u8 = 3;
pub const STT_FILE: u8 = 4;

// Special section indices
pub const SHN_UNDEF: u16 = 0;
pub const SHN_ABS: u16 = 0xFFF1;
pub const SHN_COMMON: u16 = 0xFFF2;

/// Parsed ELF64 symbol table entry.
#[derive(Debug, Clone, Copy)]
pub struct Elf64Sym {
    pub name: u32,
    pub info: u8,
    pub other: u8,
    pub shndx: u16,
    pub value: u64,
    pub size: u64,
}

impl Elf64Sym {
    /// Symbol binding (local, global, weak).
    pub fn binding(&self) -> u8 {
        self.info >> 4
    }

    /// Symbol type (notype, object, func, section, file).
    pub fn sym_type(&self) -> u8 {
        self.info & 0xF
    }

    /// Is this an undefined symbol?
    pub fn is_undefined(&self) -> bool {
        self.shndx == SHN_UNDEF
    }
}

/// Parse a single ELF64 symbol at the given byte offset.
#[must_use]
pub fn parse_elf64_sym(data: &[u8], offset: usize) -> Result<Elf64Sym, ElfError> {
    if offset + 24 > data.len() {
        return Err(ElfError::SectionOutOfBounds);
    }

    Ok(Elf64Sym {
        name: read_u32_le(data, offset),
        info: data[offset + 4],
        other: data[offset + 5],
        shndx: read_u16_le(data, offset + 6),
        value: read_u64_le(data, offset + 8),
        size: read_u64_le(data, offset + 16),
    })
}

/// Iterator over symbols in a symbol table section.
pub struct SymbolIter<'a> {
    data: &'a [u8],
    offset: usize,
    end: usize,
}

impl<'a> SymbolIter<'a> {
    /// Create from raw symtab section data.
    pub fn new(symtab_data: &'a [u8]) -> Self {
        Self {
            data: symtab_data,
            offset: 0,
            end: symtab_data.len(),
        }
    }
}

impl<'a> Iterator for SymbolIter<'a> {
    type Item = Elf64Sym;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + 24 > self.end {
            return None;
        }
        let sym = parse_elf64_sym(self.data, self.offset).ok()?;
        self.offset += 24;
        Some(sym)
    }
}

/// Find a symbol by name in a symtab + strtab pair.
#[must_use]
pub fn find_symbol_by_name<'a>(
    symtab_data: &'a [u8],
    strtab_data: &'a [u8],
    name: &str,
) -> Option<Elf64Sym> {
    for sym in SymbolIter::new(symtab_data) {
        if let Some(sym_name) = strtab_lookup(strtab_data, sym.name) {
            if sym_name == name {
                return Some(sym);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sym(name: u32, info: u8, shndx: u16, value: u64, size: u64) -> [u8; 24] {
        let mut buf = [0u8; 24];
        buf[0..4].copy_from_slice(&name.to_le_bytes());
        buf[4] = info;
        buf[5] = 0; // other
        buf[6..8].copy_from_slice(&shndx.to_le_bytes());
        buf[8..16].copy_from_slice(&value.to_le_bytes());
        buf[16..24].copy_from_slice(&size.to_le_bytes());
        buf
    }

    #[test]
    fn test_parse_sym() {
        let info = (STB_GLOBAL << 4) | STT_FUNC;
        let data = make_sym(5, info, 1, 0x401000, 64);
        let sym = parse_elf64_sym(&data, 0).unwrap();
        assert_eq!(sym.name, 5);
        assert_eq!(sym.binding(), STB_GLOBAL);
        assert_eq!(sym.sym_type(), STT_FUNC);
        assert_eq!(sym.shndx, 1);
        assert_eq!(sym.value, 0x401000);
        assert_eq!(sym.size, 64);
        assert!(!sym.is_undefined());
    }

    #[test]
    fn test_undefined_sym() {
        let data = make_sym(0, 0, SHN_UNDEF, 0, 0);
        let sym = parse_elf64_sym(&data, 0).unwrap();
        assert!(sym.is_undefined());
    }

    #[test]
    fn test_symbol_iter() {
        let mut data = [0u8; 48];
        let s1 = make_sym(0, 0, 0, 0, 0);
        let s2 = make_sym(1, (STB_GLOBAL << 4) | STT_FUNC, 1, 0x1000, 32);
        data[0..24].copy_from_slice(&s1);
        data[24..48].copy_from_slice(&s2);

        let syms: Vec<_> = SymbolIter::new(&data).collect();
        assert_eq!(syms.len(), 2);
        assert_eq!(syms[1].value, 0x1000);
    }

    #[test]
    fn test_find_by_name() {
        let strtab = b"\0main\0helper\0";
        let info = (STB_GLOBAL << 4) | STT_FUNC;
        let mut symtab = [0u8; 48];
        symtab[0..24].copy_from_slice(&make_sym(0, 0, 0, 0, 0)); // null entry
        symtab[24..48].copy_from_slice(&make_sym(1, info, 1, 0x2000, 100));

        let sym = find_symbol_by_name(&symtab, strtab, "main").unwrap();
        assert_eq!(sym.value, 0x2000);
        assert_eq!(sym.size, 100);

        assert!(find_symbol_by_name(&symtab, strtab, "nonexistent").is_none());
    }
}

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
use alloc::vec::Vec;
