//! ACPI RSDP and MADT parser (x86).
//!
//! Parses tables from byte slices (caller provides the mapped memory).
//! No heap, no I/O, no MMIO access.

/// RSDP signature: "RSD PTR "
pub const RSDP_SIGNATURE: [u8; 8] = *b"RSD PTR ";

/// MADT signature: "APIC"
pub const MADT_SIGNATURE: [u8; 4] = *b"APIC";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpiError {
    TooShort,
    BadSignature,
    BadChecksum,
    OutOfBounds,
    NotFound,
}

/// Parsed RSDP (Root System Description Pointer).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rsdp {
    pub revision: u8,
    pub rsdt_address: u32, // ACPI 1.0
    pub xsdt_address: u64, // ACPI 2.0+ (only if revision >= 2)
    pub is_xsdt_valid: bool,
}

fn read_u32_le(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

fn read_u64_le(data: &[u8], off: usize) -> u64 {
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

/// Compute ACPI checksum over a range. Valid if sum == 0 mod 256.
fn acpi_checksum(data: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    for &b in data {
        sum = sum.wrapping_add(b);
    }
    sum
}

/// Parse an RSDP from a byte slice (20 or 36 bytes).
pub fn parse_rsdp(data: &[u8]) -> Result<Rsdp, AcpiError> {
    if data.len() < 20 {
        return Err(AcpiError::TooShort);
    }
    if data[0..8] != RSDP_SIGNATURE {
        return Err(AcpiError::BadSignature);
    }
    if acpi_checksum(&data[..20]) != 0 {
        return Err(AcpiError::BadChecksum);
    }

    let revision = data[15];
    let rsdt_address = read_u32_le(data, 16);

    let (xsdt_address, is_xsdt_valid) = if revision >= 2 && data.len() >= 36 {
        if acpi_checksum(&data[..36]) != 0 {
            return Err(AcpiError::BadChecksum);
        }
        (read_u64_le(data, 24), true)
    } else {
        (0, false)
    };

    Ok(Rsdp {
        revision,
        rsdt_address,
        xsdt_address,
        is_xsdt_valid,
    })
}

/// Search for RSDP in a memory region (scans for signature).
/// RSDP is 16-byte aligned in EBDA or 0xE0000–0xFFFFF.
pub fn find_rsdp(region: &[u8]) -> Result<Rsdp, AcpiError> {
    let mut off = 0;
    while off + 20 <= region.len() {
        if region[off..off + 8] == RSDP_SIGNATURE {
            if let Ok(rsdp) = parse_rsdp(&region[off..]) {
                return Ok(rsdp);
            }
        }
        off += 16; // RSDP is always 16-byte aligned
    }
    Err(AcpiError::NotFound)
}

// ── MADT (Multiple APIC Description Table) ──────────────────

/// MADT entry types.
pub const MADT_LOCAL_APIC: u8 = 0;
pub const MADT_IO_APIC: u8 = 1;
pub const MADT_INTERRUPT_OVERRIDE: u8 = 2;
pub const MADT_LOCAL_APIC_NMI: u8 = 4;

/// Parsed MADT header.
#[derive(Debug, Clone, Copy)]
pub struct MadtHeader {
    pub local_apic_address: u32,
    pub flags: u32,
}

/// A single MADT entry.
#[derive(Debug, Clone, Copy)]
pub enum MadtEntry {
    LocalApic {
        acpi_processor_id: u8,
        apic_id: u8,
        flags: u32,
    },
    IoApic {
        io_apic_id: u8,
        io_apic_address: u32,
        global_system_interrupt_base: u32,
    },
    InterruptOverride {
        bus: u8,
        source: u8,
        global_system_interrupt: u32,
        flags: u16,
    },
    LocalApicNmi {
        acpi_processor_id: u8,
        flags: u16,
        lint: u8,
    },
    Unknown {
        entry_type: u8,
        length: u8,
    },
}

/// Parse the MADT from a raw ACPI table byte slice.
/// The slice should start at the ACPI table header (signature at offset 0).
pub fn parse_madt(data: &[u8]) -> Result<(MadtHeader, MadtEntryIter<'_>), AcpiError> {
    if data.len() < 44 {
        return Err(AcpiError::TooShort);
    }
    if data[0..4] != MADT_SIGNATURE {
        return Err(AcpiError::BadSignature);
    }

    let table_len = read_u32_le(data, 4) as usize;
    if table_len > data.len() {
        return Err(AcpiError::OutOfBounds);
    }

    if acpi_checksum(&data[..table_len]) != 0 {
        return Err(AcpiError::BadChecksum);
    }

    let header = MadtHeader {
        local_apic_address: read_u32_le(data, 36),
        flags: read_u32_le(data, 40),
    };

    Ok((
        header,
        MadtEntryIter {
            data,
            offset: 44,
            end: table_len,
        },
    ))
}

/// Iterator over MADT entries.
pub struct MadtEntryIter<'a> {
    data: &'a [u8],
    offset: usize,
    end: usize,
}

impl<'a> Iterator for MadtEntryIter<'a> {
    type Item = MadtEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + 2 > self.end {
            return None;
        }
        let entry_type = self.data[self.offset];
        let length = self.data[self.offset + 1] as usize;
        if length < 2 || self.offset + length > self.end {
            return None;
        }

        let d = &self.data[self.offset..self.offset + length];
        self.offset += length;

        Some(match entry_type {
            MADT_LOCAL_APIC if length >= 8 => MadtEntry::LocalApic {
                acpi_processor_id: d[2],
                apic_id: d[3],
                flags: read_u32_le(d, 4),
            },
            MADT_IO_APIC if length >= 12 => MadtEntry::IoApic {
                io_apic_id: d[2],
                io_apic_address: read_u32_le(d, 4),
                global_system_interrupt_base: read_u32_le(d, 8),
            },
            MADT_INTERRUPT_OVERRIDE if length >= 10 => MadtEntry::InterruptOverride {
                bus: d[2],
                source: d[3],
                global_system_interrupt: read_u32_le(d, 4),
                flags: u16::from_le_bytes([d[8], d[9]]),
            },
            MADT_LOCAL_APIC_NMI if length >= 6 => MadtEntry::LocalApicNmi {
                acpi_processor_id: d[2],
                flags: u16::from_le_bytes([d[3], d[4]]),
                lint: d[5],
            },
            _ => MadtEntry::Unknown {
                entry_type,
                length: length as u8,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rsdp_v1(rsdt_addr: u32) -> [u8; 20] {
        let mut buf = [0u8; 20];
        buf[0..8].copy_from_slice(&RSDP_SIGNATURE);
        buf[15] = 0; // revision 0
        buf[16..20].copy_from_slice(&rsdt_addr.to_le_bytes());
        // Fix checksum
        let sum: u8 = buf.iter().fold(0u8, |a, &b| a.wrapping_add(b));
        buf[8] = buf[8].wrapping_sub(sum); // checksum field at offset 8
        buf
    }

    #[test]
    fn test_parse_rsdp_v1() {
        let data = make_rsdp_v1(0xDEAD0000);
        let rsdp = parse_rsdp(&data).unwrap();
        assert_eq!(rsdp.revision, 0);
        assert_eq!(rsdp.rsdt_address, 0xDEAD0000);
        assert!(!rsdp.is_xsdt_valid);
    }

    #[test]
    fn test_rsdp_bad_signature() {
        let mut data = make_rsdp_v1(0);
        data[0] = b'X';
        assert_eq!(parse_rsdp(&data), Err(AcpiError::BadSignature));
    }

    #[test]
    fn test_rsdp_bad_checksum() {
        let mut data = make_rsdp_v1(0);
        data[9] = 0xFF; // corrupt data
        assert_eq!(parse_rsdp(&data), Err(AcpiError::BadChecksum));
    }

    #[test]
    fn test_find_rsdp() {
        let mut region = [0u8; 256];
        // Place RSDP at offset 48 (16-byte aligned)
        let rsdp = make_rsdp_v1(0x12345678);
        region[48..68].copy_from_slice(&rsdp);

        let found = find_rsdp(&region).unwrap();
        assert_eq!(found.rsdt_address, 0x12345678);
    }

    #[test]
    fn test_find_rsdp_not_found() {
        let region = [0u8; 256];
        assert_eq!(find_rsdp(&region), Err(AcpiError::NotFound));
    }

    fn build_madt(entries: &[u8]) -> alloc::vec::Vec<u8> {
        let table_len = 44 + entries.len();
        let mut buf = alloc::vec::Vec::new();
        // Signature "APIC"
        buf.extend_from_slice(&MADT_SIGNATURE);
        // Length
        buf.extend_from_slice(&(table_len as u32).to_le_bytes());
        // Revision
        buf.push(3);
        // Checksum placeholder
        buf.push(0);
        // OEMID (6), OEM table ID (8), OEM rev (4), Creator ID (4), Creator Rev (4)
        buf.extend_from_slice(&[0u8; 26]);
        // Local APIC address
        buf.extend_from_slice(&0xFEE00000u32.to_le_bytes());
        // Flags
        buf.extend_from_slice(&1u32.to_le_bytes());
        // Entries
        buf.extend_from_slice(entries);
        // Fix checksum
        let sum: u8 = buf.iter().fold(0u8, |a, &b| a.wrapping_add(b));
        buf[9] = buf[9].wrapping_sub(sum);
        buf
    }

    #[test]
    fn test_parse_madt_local_apic() {
        // Local APIC entry: type=0, length=8
        let entry = [0, 8, 0, 1, 1, 0, 0, 0]; // proc_id=0, apic_id=1, flags=1
        let table = build_madt(&entry);
        let (hdr, mut iter) = parse_madt(&table).unwrap();
        assert_eq!(hdr.local_apic_address, 0xFEE00000);

        match iter.next().unwrap() {
            MadtEntry::LocalApic {
                acpi_processor_id,
                apic_id,
                flags,
            } => {
                assert_eq!(acpi_processor_id, 0);
                assert_eq!(apic_id, 1);
                assert_eq!(flags, 1);
            }
            _ => panic!("Expected LocalApic"),
        }
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_parse_madt_io_apic() {
        // IO APIC entry: type=1, length=12
        let mut entry = [0u8; 12];
        entry[0] = 1; // type
        entry[1] = 12; // length
        entry[2] = 2; // io_apic_id
        entry[4..8].copy_from_slice(&0xFEC00000u32.to_le_bytes());
        entry[8..12].copy_from_slice(&0u32.to_le_bytes());

        let table = build_madt(&entry);
        let (_, mut iter) = parse_madt(&table).unwrap();

        match iter.next().unwrap() {
            MadtEntry::IoApic {
                io_apic_id,
                io_apic_address,
                global_system_interrupt_base,
            } => {
                assert_eq!(io_apic_id, 2);
                assert_eq!(io_apic_address, 0xFEC00000);
                assert_eq!(global_system_interrupt_base, 0);
            }
            _ => panic!("Expected IoApic"),
        }
    }
}

#[cfg(test)]
extern crate alloc;
