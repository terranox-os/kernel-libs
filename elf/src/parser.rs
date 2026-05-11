//! ELF header, section header, and program header parsing.

/// ELF magic bytes.
pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

// ELF class
pub const ELFCLASS32: u8 = 1;
pub const ELFCLASS64: u8 = 2;

// ELF data encoding
pub const ELFDATA2LSB: u8 = 1;
pub const ELFDATA2MSB: u8 = 2;

// ELF type
pub const ET_NONE: u16 = 0;
pub const ET_REL: u16 = 1;
pub const ET_EXEC: u16 = 2;
pub const ET_DYN: u16 = 3;

// ELF machine
pub const EM_386: u16 = 3;
pub const EM_ARM: u16 = 40;
pub const EM_X86_64: u16 = 62;
pub const EM_AARCH64: u16 = 183;
pub const EM_RISCV: u16 = 243;

// Section types
pub const SHT_NULL: u32 = 0;
pub const SHT_PROGBITS: u32 = 1;
pub const SHT_SYMTAB: u32 = 2;
pub const SHT_STRTAB: u32 = 3;
pub const SHT_RELA: u32 = 4;
pub const SHT_NOBITS: u32 = 8;
pub const SHT_REL: u32 = 9;
pub const SHT_DYNSYM: u32 = 11;

// Section flags
pub const SHF_WRITE: u64 = 0x1;
pub const SHF_ALLOC: u64 = 0x2;
pub const SHF_EXECINSTR: u64 = 0x4;

// Program header types
pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PT_NOTE: u32 = 4;
pub const PT_PHDR: u32 = 6;

// Program header flags
pub const PF_X: u32 = 0x1;
pub const PF_W: u32 = 0x2;
pub const PF_R: u32 = 0x4;

/// Error type for ELF parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfError {
    TooShort,
    BadMagic,
    UnsupportedClass,
    UnsupportedEncoding,
    SectionOutOfBounds,
    SegmentOutOfBounds,
    InvalidIndex,
}

/// Parsed ELF64 file header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf64Header {
    pub class: u8,
    pub data: u8,
    pub elf_type: u16,
    pub machine: u16,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

/// Parsed ELF64 section header.
#[derive(Debug, Clone, Copy)]
pub struct Elf64Shdr {
    pub name: u32,
    pub sh_type: u32,
    pub flags: u64,
    pub addr: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub addralign: u64,
    pub entsize: u64,
}

/// Parsed ELF64 program header.
#[derive(Debug, Clone, Copy)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub flags: u32,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

/// Helper to read little-endian integers from a byte slice.
pub(crate) fn read_u16_le(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off + 1]])
}

pub(crate) fn read_u32_le(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

pub(crate) fn read_u64_le(data: &[u8], off: usize) -> u64 {
    u64::from_le_bytes([
        data[off],
        data[off + 1],
        data[off + 2],
        data[off + 3],
        data[off + 4],
        data[off + 5],
        data[off + 6],
        data[off + 7],
    ])
}

/// Parse an ELF64 file header from a byte slice.
pub fn parse_elf64_header(data: &[u8]) -> Result<Elf64Header, ElfError> {
    if data.len() < 64 {
        return Err(ElfError::TooShort);
    }
    if data[0..4] != ELF_MAGIC {
        return Err(ElfError::BadMagic);
    }
    if data[4] != ELFCLASS64 {
        return Err(ElfError::UnsupportedClass);
    }
    if data[5] != ELFDATA2LSB {
        return Err(ElfError::UnsupportedEncoding);
    }

    Ok(Elf64Header {
        class: data[4],
        data: data[5],
        elf_type: read_u16_le(data, 16),
        machine: read_u16_le(data, 18),
        entry: read_u64_le(data, 24),
        phoff: read_u64_le(data, 32),
        shoff: read_u64_le(data, 40),
        phentsize: read_u16_le(data, 54),
        phnum: read_u16_le(data, 56),
        shentsize: read_u16_le(data, 58),
        shnum: read_u16_le(data, 60),
        shstrndx: read_u16_le(data, 62),
    })
}

/// Parse an ELF64 section header at the given index.
pub fn parse_elf64_shdr(data: &[u8], hdr: &Elf64Header, index: u16) -> Result<Elf64Shdr, ElfError> {
    if index >= hdr.shnum {
        return Err(ElfError::InvalidIndex);
    }
    let off = hdr.shoff as usize + (index as usize) * (hdr.shentsize as usize);
    if off + 64 > data.len() {
        return Err(ElfError::SectionOutOfBounds);
    }

    Ok(Elf64Shdr {
        name: read_u32_le(data, off),
        sh_type: read_u32_le(data, off + 4),
        flags: read_u64_le(data, off + 8),
        addr: read_u64_le(data, off + 16),
        offset: read_u64_le(data, off + 24),
        size: read_u64_le(data, off + 32),
        link: read_u32_le(data, off + 40),
        info: read_u32_le(data, off + 44),
        addralign: read_u64_le(data, off + 48),
        entsize: read_u64_le(data, off + 56),
    })
}

/// Parse an ELF64 program header at the given index.
pub fn parse_elf64_phdr(data: &[u8], hdr: &Elf64Header, index: u16) -> Result<Elf64Phdr, ElfError> {
    if index >= hdr.phnum {
        return Err(ElfError::InvalidIndex);
    }
    let off = hdr.phoff as usize + (index as usize) * (hdr.phentsize as usize);
    if off + 56 > data.len() {
        return Err(ElfError::SegmentOutOfBounds);
    }

    Ok(Elf64Phdr {
        p_type: read_u32_le(data, off),
        flags: read_u32_le(data, off + 4),
        offset: read_u64_le(data, off + 8),
        vaddr: read_u64_le(data, off + 16),
        paddr: read_u64_le(data, off + 24),
        filesz: read_u64_le(data, off + 32),
        memsz: read_u64_le(data, off + 40),
        align: read_u64_le(data, off + 48),
    })
}

/// Get a section's raw data from the ELF file.
pub fn section_data<'a>(data: &'a [u8], shdr: &Elf64Shdr) -> Result<&'a [u8], ElfError> {
    if shdr.sh_type == SHT_NOBITS {
        return Ok(&[]);
    }
    let start = shdr.offset as usize;
    let end = start + shdr.size as usize;
    if end > data.len() {
        return Err(ElfError::SectionOutOfBounds);
    }
    Ok(&data[start..end])
}

/// Look up a null-terminated string in a string table section.
pub fn strtab_lookup(strtab_data: &[u8], offset: u32) -> Option<&str> {
    let start = offset as usize;
    if start >= strtab_data.len() {
        return None;
    }
    let remaining = &strtab_data[start..];
    let len = remaining.iter().position(|&b| b == 0)?;
    core::str::from_utf8(&remaining[..len]).ok()
}

/// Iterate over section headers.
pub struct SectionIter<'a> {
    data: &'a [u8],
    hdr: Elf64Header,
    index: u16,
}

impl<'a> SectionIter<'a> {
    pub fn new(data: &'a [u8], hdr: Elf64Header) -> Self {
        Self {
            data,
            hdr,
            index: 0,
        }
    }
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = (u16, Elf64Shdr);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.hdr.shnum {
            return None;
        }
        let idx = self.index;
        self.index += 1;
        parse_elf64_shdr(self.data, &self.hdr, idx)
            .ok()
            .map(|shdr| (idx, shdr))
    }
}

/// Iterate over program headers.
pub struct SegmentIter<'a> {
    data: &'a [u8],
    hdr: Elf64Header,
    index: u16,
}

impl<'a> SegmentIter<'a> {
    pub fn new(data: &'a [u8], hdr: Elf64Header) -> Self {
        Self {
            data,
            hdr,
            index: 0,
        }
    }
}

impl<'a> Iterator for SegmentIter<'a> {
    type Item = Elf64Phdr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.hdr.phnum {
            return None;
        }
        let idx = self.index;
        self.index += 1;
        parse_elf64_phdr(self.data, &self.hdr, idx).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal ELF64 header (little-endian, x86-64, no sections/segments)
    fn minimal_elf64() -> [u8; 64] {
        let mut buf = [0u8; 64];
        // Magic
        buf[0..4].copy_from_slice(&ELF_MAGIC);
        buf[4] = ELFCLASS64; // class
        buf[5] = ELFDATA2LSB; // data encoding
        buf[6] = 1; // ELF version
                    // e_type = ET_EXEC
        buf[16] = ET_EXEC as u8;
        buf[17] = (ET_EXEC >> 8) as u8;
        // e_machine = EM_X86_64
        buf[18] = EM_X86_64 as u8;
        buf[19] = (EM_X86_64 >> 8) as u8;
        // e_entry = 0x400000
        buf[24..32].copy_from_slice(&0x400000u64.to_le_bytes());
        // e_phentsize = 56
        buf[54..56].copy_from_slice(&56u16.to_le_bytes());
        // e_shentsize = 64
        buf[58..60].copy_from_slice(&64u16.to_le_bytes());
        buf
    }

    #[test]
    fn test_parse_minimal_header() {
        let data = minimal_elf64();
        let hdr = parse_elf64_header(&data).unwrap();
        assert_eq!(hdr.class, ELFCLASS64);
        assert_eq!(hdr.data, ELFDATA2LSB);
        assert_eq!(hdr.elf_type, ET_EXEC);
        assert_eq!(hdr.machine, EM_X86_64);
        assert_eq!(hdr.entry, 0x400000);
        assert_eq!(hdr.phnum, 0);
        assert_eq!(hdr.shnum, 0);
    }

    #[test]
    fn test_bad_magic() {
        let mut data = minimal_elf64();
        data[0] = 0;
        assert_eq!(parse_elf64_header(&data), Err(ElfError::BadMagic));
    }

    #[test]
    fn test_too_short() {
        let data = [0u8; 32];
        assert_eq!(parse_elf64_header(&data), Err(ElfError::TooShort));
    }

    #[test]
    fn test_wrong_class() {
        let mut data = minimal_elf64();
        data[4] = ELFCLASS32;
        assert_eq!(parse_elf64_header(&data), Err(ElfError::UnsupportedClass));
    }

    #[test]
    fn test_strtab_lookup() {
        let strtab = b"\0hello\0world\0";
        assert_eq!(strtab_lookup(strtab, 0), Some(""));
        assert_eq!(strtab_lookup(strtab, 1), Some("hello"));
        assert_eq!(strtab_lookup(strtab, 7), Some("world"));
        assert_eq!(strtab_lookup(strtab, 100), None);
    }

    #[test]
    fn test_section_iter_empty() {
        let data = minimal_elf64();
        let hdr = parse_elf64_header(&data).unwrap();
        let sections: Vec<_> = SectionIter::new(&data, hdr).collect();
        assert!(sections.is_empty());
    }
}

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
use alloc::vec::Vec;
