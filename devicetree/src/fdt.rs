//! Flattened Device Tree (FDT / DTB) parser.
//!
//! Parses an FDT blob from a byte slice. No heap, no I/O.
//! Used by GenesisOS-RT (ARM) and HermeticaOS (multi-arch).

/// FDT magic number.
pub const FDT_MAGIC: u32 = 0xD00DFEED;

// FDT structure tokens
const FDT_BEGIN_NODE: u32 = 0x00000001;
const FDT_END_NODE: u32 = 0x00000002;
const FDT_PROP: u32 = 0x00000003;
const FDT_NOP: u32 = 0x00000004;
const FDT_END: u32 = 0x00000009;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdtError {
    TooShort,
    BadMagic,
    OutOfBounds,
    NotFound,
}

/// Parsed FDT header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FdtHeader {
    pub magic: u32,
    pub totalsize: u32,
    pub off_dt_struct: u32,
    pub off_dt_strings: u32,
    pub off_mem_rsvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32,
}

fn read_u32_be(data: &[u8], off: usize) -> u32 {
    u32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

fn read_u64_be(data: &[u8], off: usize) -> u64 {
    u64::from_be_bytes([
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

/// Align offset up to 4-byte boundary.
fn align4(off: usize) -> usize {
    (off + 3) & !3
}

/// Parse the FDT header from a DTB blob.
pub fn parse_fdt_header(data: &[u8]) -> Result<FdtHeader, FdtError> {
    if data.len() < 40 {
        return Err(FdtError::TooShort);
    }
    let magic = read_u32_be(data, 0);
    if magic != FDT_MAGIC {
        return Err(FdtError::BadMagic);
    }

    Ok(FdtHeader {
        magic,
        totalsize: read_u32_be(data, 4),
        off_dt_struct: read_u32_be(data, 8),
        off_dt_strings: read_u32_be(data, 12),
        off_mem_rsvmap: read_u32_be(data, 16),
        version: read_u32_be(data, 20),
        last_comp_version: read_u32_be(data, 24),
        boot_cpuid_phys: read_u32_be(data, 28),
        size_dt_strings: read_u32_be(data, 32),
        size_dt_struct: read_u32_be(data, 36),
    })
}

/// Look up a string in the FDT strings block.
fn fdt_string<'a>(data: &'a [u8], hdr: &FdtHeader, nameoff: u32) -> Option<&'a str> {
    let start = hdr.off_dt_strings as usize + nameoff as usize;
    if start >= data.len() {
        return None;
    }
    let remaining = &data[start..];
    let len = remaining.iter().position(|&b| b == 0)?;
    core::str::from_utf8(&remaining[..len]).ok()
}

/// A property found during FDT traversal.
#[derive(Debug, Clone, Copy)]
pub struct FdtProperty<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

impl<'a> FdtProperty<'a> {
    /// Read property as a u32 (big-endian).
    pub fn as_u32(&self) -> Option<u32> {
        if self.data.len() >= 4 {
            Some(read_u32_be(self.data, 0))
        } else {
            None
        }
    }

    /// Read property as a u64 (big-endian).
    pub fn as_u64(&self) -> Option<u64> {
        if self.data.len() >= 8 {
            Some(read_u64_be(self.data, 0))
        } else {
            None
        }
    }

    /// Read property as a string.
    pub fn as_str(&self) -> Option<&'a str> {
        let len = self
            .data
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.data.len());
        core::str::from_utf8(&self.data[..len]).ok()
    }
}

/// Find a node by path (e.g., "/memory" or "/cpus/cpu@0") and invoke
/// a callback for each property in that node.
///
/// Returns the number of properties found, or an error.
pub fn find_node_properties<'a>(
    data: &'a [u8],
    hdr: &FdtHeader,
    path: &str,
) -> Result<FdtPropertyList<'a>, FdtError> {
    let struct_start = hdr.off_dt_struct as usize;
    let struct_end = struct_start + hdr.size_dt_struct as usize;

    if struct_end > data.len() {
        return Err(FdtError::OutOfBounds);
    }

    // Split the path into components
    let target_parts = PathParts::from_path(path);

    let mut offset = struct_start;
    let mut depth: i32 = 0;
    let mut current_path_depth: usize = 0;
    // If path is "/" (no parts), target is the root node
    let mut in_target = target_parts.len() == 0;
    let mut root_entered = false;
    let mut props = FdtPropertyList::new();

    while offset + 4 <= struct_end {
        let token = read_u32_be(data, offset);
        offset += 4;

        match token {
            FDT_BEGIN_NODE => {
                // Read node name (null-terminated, 4-byte aligned)
                let name_start = offset;
                while offset < struct_end && data[offset] != 0 {
                    offset += 1;
                }
                let name = core::str::from_utf8(&data[name_start..offset]).unwrap_or("");
                offset = align4(offset + 1); // skip null + align

                if !in_target
                    && current_path_depth < target_parts.len()
                    && node_name_matches(name, &target_parts[current_path_depth])
                {
                    current_path_depth += 1;
                    if current_path_depth == target_parts.len() {
                        in_target = true;
                    }
                }
                if in_target && !root_entered {
                    root_entered = true;
                }
                depth += 1;
            }
            FDT_END_NODE => {
                if in_target {
                    return Ok(props);
                }
                depth -= 1;
                if depth < 0 {
                    break;
                }
                if current_path_depth > depth as usize {
                    current_path_depth = depth as usize;
                }
            }
            FDT_PROP => {
                if offset + 8 > struct_end {
                    return Err(FdtError::OutOfBounds);
                }
                let len = read_u32_be(data, offset) as usize;
                let nameoff = read_u32_be(data, offset + 4);
                offset += 8;

                let prop_data = if offset + len <= struct_end {
                    &data[offset..offset + len]
                } else {
                    return Err(FdtError::OutOfBounds);
                };
                offset = align4(offset + len);

                if in_target {
                    if let Some(name) = fdt_string(data, hdr, nameoff) {
                        props.push(FdtProperty {
                            name,
                            data: prop_data,
                        });
                    }
                }
            }
            FDT_NOP => {}
            FDT_END => break,
            _ => break, // Unknown token
        }
    }

    if in_target {
        Ok(props)
    } else {
        Err(FdtError::NotFound)
    }
}

/// Check if node name matches target (ignoring @unit-address).
fn node_name_matches(node_name: &str, target: &str) -> bool {
    if node_name == target {
        return true;
    }
    // Match "cpu@0" against "cpu@0", or "memory" against "memory"
    if let Some(base) = node_name.split('@').next() {
        if let Some(target_base) = target.split('@').next() {
            if target.contains('@') {
                return node_name == target;
            }
            return base == target_base;
        }
    }
    false
}

/// Small fixed-capacity list of properties (no heap).
pub struct FdtPropertyList<'a> {
    props: [Option<FdtProperty<'a>>; 32],
    len: usize,
}

impl<'a> FdtPropertyList<'a> {
    const fn new() -> Self {
        Self {
            props: [None; 32],
            len: 0,
        }
    }

    fn push(&mut self, prop: FdtProperty<'a>) {
        if self.len < 32 {
            self.props[self.len] = Some(prop);
            self.len += 1;
        }
    }

    /// Number of properties.
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get property by index.
    pub fn get(&self, index: usize) -> Option<&FdtProperty<'a>> {
        if index < self.len {
            self.props[index].as_ref()
        } else {
            None
        }
    }

    /// Find a property by name.
    pub fn find(&self, name: &str) -> Option<&FdtProperty<'a>> {
        for i in 0..self.len {
            if let Some(ref p) = self.props[i] {
                if p.name == name {
                    return Some(p);
                }
            }
        }
        None
    }
}

/// Small fixed-capacity list for path components.
struct PathParts<'a> {
    parts: [&'a str; 16],
    len: usize,
}

impl<'a> PathParts<'a> {
    fn from_path(path: &'a str) -> Self {
        let mut p = Self {
            parts: [""; 16],
            len: 0,
        };
        for segment in path.split('/') {
            if !segment.is_empty() && p.len < 16 {
                p.parts[p.len] = segment;
                p.len += 1;
            }
        }
        p
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<'a> core::ops::Index<usize> for PathParts<'a> {
    type Output = str;
    fn index(&self, idx: usize) -> &str {
        self.parts[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal FDT blob for testing.
    fn build_test_fdt() -> alloc::vec::Vec<u8> {
        let mut blob = alloc::vec::Vec::new();

        // We'll build: header + memory reservation + struct + strings
        let strings = b"model\0compatible\0#address-cells\0";
        let _strings_off: u32 = 0; // calculated below

        // Build structure block
        let mut structure = alloc::vec::Vec::new();
        // FDT_BEGIN_NODE (root "")
        push_u32_be(&mut structure, FDT_BEGIN_NODE);
        structure.push(0); // empty root name
        pad4(&mut structure);

        // Property: model = "test-board"
        push_u32_be(&mut structure, FDT_PROP);
        let model_val = b"test-board\0";
        push_u32_be(&mut structure, model_val.len() as u32);
        push_u32_be(&mut structure, 0); // nameoff for "model"
        structure.extend_from_slice(model_val);
        pad4(&mut structure);

        // Property: compatible = "test,soc"
        push_u32_be(&mut structure, FDT_PROP);
        let compat_val = b"test,soc\0";
        push_u32_be(&mut structure, compat_val.len() as u32);
        push_u32_be(&mut structure, 6); // nameoff for "compatible"
        structure.extend_from_slice(compat_val);
        pad4(&mut structure);

        // FDT_END_NODE (root)
        push_u32_be(&mut structure, FDT_END_NODE);
        push_u32_be(&mut structure, FDT_END);

        // Calculate offsets
        let header_size = 40u32;
        let mem_rsvmap_off = header_size;
        let mem_rsvmap_size = 16u32; // one empty entry (16 bytes of zeros)
        let struct_off = mem_rsvmap_off + mem_rsvmap_size;
        let struct_size = structure.len() as u32;
        let strings_off_val = struct_off + struct_size;
        let total = strings_off_val + strings.len() as u32;

        // Header
        push_u32_be(&mut blob, FDT_MAGIC);
        push_u32_be(&mut blob, total);
        push_u32_be(&mut blob, struct_off);
        push_u32_be(&mut blob, strings_off_val);
        push_u32_be(&mut blob, mem_rsvmap_off);
        push_u32_be(&mut blob, 17); // version
        push_u32_be(&mut blob, 16); // last_comp_version
        push_u32_be(&mut blob, 0); // boot_cpuid_phys
        push_u32_be(&mut blob, strings.len() as u32);
        push_u32_be(&mut blob, struct_size);

        // Memory reservation (empty terminator)
        blob.extend_from_slice(&[0u8; 16]);

        // Structure
        blob.extend_from_slice(&structure);

        // Strings
        blob.extend_from_slice(strings);

        blob
    }

    fn push_u32_be(buf: &mut alloc::vec::Vec<u8>, val: u32) {
        buf.extend_from_slice(&val.to_be_bytes());
    }

    fn pad4(buf: &mut alloc::vec::Vec<u8>) {
        while buf.len() % 4 != 0 {
            buf.push(0);
        }
    }

    #[test]
    fn test_parse_header() {
        let fdt = build_test_fdt();
        let hdr = parse_fdt_header(&fdt).unwrap();
        assert_eq!(hdr.magic, FDT_MAGIC);
        assert_eq!(hdr.version, 17);
    }

    #[test]
    fn test_bad_magic() {
        let mut fdt = build_test_fdt();
        fdt[0] = 0;
        assert_eq!(parse_fdt_header(&fdt), Err(FdtError::BadMagic));
    }

    #[test]
    fn test_find_root_properties() {
        let fdt = build_test_fdt();
        let hdr = parse_fdt_header(&fdt).unwrap();
        let props = find_node_properties(&fdt, &hdr, "/").unwrap();
        assert!(props.len() >= 2);

        let model = props.find("model").unwrap();
        assert_eq!(model.as_str(), Some("test-board"));

        let compat = props.find("compatible").unwrap();
        assert_eq!(compat.as_str(), Some("test,soc"));
    }

    #[test]
    fn test_node_not_found() {
        let fdt = build_test_fdt();
        let hdr = parse_fdt_header(&fdt).unwrap();
        assert!(matches!(
            find_node_properties(&fdt, &hdr, "/nonexistent"),
            Err(FdtError::NotFound)
        ));
    }
}

#[cfg(test)]
extern crate alloc;
