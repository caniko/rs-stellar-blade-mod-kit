fn probe_pak(path: &Path) -> Result<PakProbe, PakProbeError> {
    let mut file = File::open(path).map_err(|source| PakProbeError::Io {
        path: path.to_path_buf(),
        action: "open pak",
        source,
    })?;
    let file_size = file
        .metadata()
        .map_err(|source| PakProbeError::Io {
            path: path.to_path_buf(),
            action: "read pak metadata",
            source,
        })?
        .len();
    let footer = read_footer(&mut file, path, file_size)?;

    if footer.index_offset >= file_size || footer.index_offset + footer.index_size > file_size {
        return Err(PakProbeError::Unsupported(format!(
            "pak index range is outside file bounds: offset={} size={} file_size={}",
            footer.index_offset, footer.index_size, file_size
        )));
    }

    file.seek(SeekFrom::Start(footer.index_offset))
        .map_err(|source| PakProbeError::Io {
            path: path.to_path_buf(),
            action: "seek to pak index",
            source,
        })?;
    let mut index = vec![0; footer.index_size as usize];
    file.read_exact(&mut index)
        .map_err(|source| PakProbeError::Io {
            path: path.to_path_buf(),
            action: "read pak index",
            source,
        })?;

    let (index_prefix_seed, mount_point, entries) = parse_index(&index)?;
    Ok(PakProbe {
        pak_path: path.to_path_buf(),
        file_size,
        footer,
        index_prefix_seed,
        mount_point,
        entries,
    })
}

fn read_footer(file: &mut File, path: &Path, file_size: u64) -> Result<PakFooter, PakProbeError> {
    let search_len = file_size.min(FOOTER_SEARCH_BYTES);
    file.seek(SeekFrom::Start(file_size - search_len))
        .map_err(|source| PakProbeError::Io {
            path: path.to_path_buf(),
            action: "seek to pak footer search window",
            source,
        })?;
    let mut bytes = vec![0; search_len as usize];
    file.read_exact(&mut bytes)
        .map_err(|source| PakProbeError::Io {
            path: path.to_path_buf(),
            action: "read pak footer search window",
            source,
        })?;

    let Some(relative_magic_offset) = bytes
        .windows(PAK_MAGIC_LE.len())
        .rposition(|window| window == PAK_MAGIC_LE)
    else {
        return Err(PakProbeError::Unsupported(
            "could not find Unreal Pak footer magic in the last 512 bytes".to_owned(),
        ));
    };
    let footer_offset = file_size - search_len + relative_magic_offset as u64;
    let footer_bytes = &bytes[relative_magic_offset..];
    let mut cursor = ByteCursor::new(footer_bytes);
    let magic = cursor.read_u32()?;
    if magic != 0x5a6f12e1 {
        return Err(PakProbeError::Unsupported(format!(
            "unexpected pak footer magic: 0x{magic:08x}"
        )));
    }
    let version = cursor.read_i32()?;
    let index_offset = cursor.read_i64()? as u64;
    let index_size = cursor.read_i64()? as u64;
    let index_hash = cursor.read_array_20()?;

    Ok(PakFooter {
        footer_offset,
        version,
        index_offset,
        index_size,
        index_hash,
    })
}

fn parse_index(index: &[u8]) -> Result<(Option<u32>, String, Vec<PakEntry>), PakProbeError> {
    if let Ok(parsed) = parse_index_at(index, 0) {
        return Ok((None, parsed.0, parsed.1));
    }
    if index.len() >= 4 {
        let seed = u32::from_le_bytes(index[0..4].try_into().expect("slice length checked"));
        if let Ok(parsed) = parse_index_at(index, 4) {
            return Ok((Some(seed), parsed.0, parsed.1));
        }
    }
    Err(PakProbeError::Unsupported(
        "unsupported pak index layout; expected optional u32 prefix, mount point FString, entry count, and uncompressed FPakEntry records".to_owned(),
    ))
}

fn parse_index_at(index: &[u8], offset: usize) -> Result<(String, Vec<PakEntry>), PakProbeError> {
    let mut cursor = ByteCursor::new(&index[offset..]);
    let mount_point = cursor.read_fstring()?;
    if mount_point.is_empty() || !mount_point.contains("../") {
        return Err(PakProbeError::Unsupported(format!(
            "unexpected pak mount point `{mount_point}`"
        )));
    }
    let entry_count = cursor.read_i32()?;
    if !(0..=100_000).contains(&entry_count) {
        return Err(PakProbeError::Unsupported(format!(
            "unsupported pak entry count `{entry_count}`"
        )));
    }

    let mut entries = Vec::with_capacity(entry_count as usize);
    for _ in 0..entry_count {
        entries.push(cursor.read_pak_entry()?);
    }
    Ok((mount_point, entries))
}

impl ByteCursor<'_> {
    fn read_pak_entry(&mut self) -> Result<PakEntry, PakProbeError> {
        let path = self.read_fstring()?;
        let offset = self.read_i64()? as u64;
        let size = self.read_i64()? as u64;
        let uncompressed_size = self.read_i64()? as u64;
        let compression_method = self.read_u32()?;
        if compression_method != 0 {
            return Err(PakProbeError::Unsupported(format!(
                "compressed pak entry `{path}` uses method index {compression_method}; compressed block parsing is not implemented"
            )));
        }
        let hash = self.read_array_20()?;
        let encrypted = self.read_u8()? != 0;
        let compression_block_size = self.read_u32()?;
        Ok(PakEntry {
            path,
            offset,
            size,
            uncompressed_size,
            compression_method,
            hash,
            encrypted,
            compression_block_size,
        })
    }
}

struct ByteCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ByteCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, PakProbeError> {
        self.require(1)?;
        let value = self.bytes[self.offset];
        self.offset += 1;
        Ok(value)
    }

    fn read_u32(&mut self) -> Result<u32, PakProbeError> {
        self.require(4)?;
        let value = u32::from_le_bytes(
            self.bytes[self.offset..self.offset + 4]
                .try_into()
                .expect("slice length checked"),
        );
        self.offset += 4;
        Ok(value)
    }

    fn read_i32(&mut self) -> Result<i32, PakProbeError> {
        self.require(4)?;
        let value = i32::from_le_bytes(
            self.bytes[self.offset..self.offset + 4]
                .try_into()
                .expect("slice length checked"),
        );
        self.offset += 4;
        Ok(value)
    }

    fn read_i64(&mut self) -> Result<i64, PakProbeError> {
        self.require(8)?;
        let value = i64::from_le_bytes(
            self.bytes[self.offset..self.offset + 8]
                .try_into()
                .expect("slice length checked"),
        );
        self.offset += 8;
        Ok(value)
    }

    fn read_array_20(&mut self) -> Result<[u8; 20], PakProbeError> {
        self.require(20)?;
        let value = self.bytes[self.offset..self.offset + 20]
            .try_into()
            .expect("slice length checked");
        self.offset += 20;
        Ok(value)
    }

    fn read_fstring(&mut self) -> Result<String, PakProbeError> {
        let length = self.read_i32()?;
        if length == 0 {
            return Ok(String::new());
        }
        if length < 0 {
            return Err(PakProbeError::Unsupported(
                "UTF-16 FString parsing is not implemented for pak index strings".to_owned(),
            ));
        }
        let length = length as usize;
        self.require(length)?;
        let raw = &self.bytes[self.offset..self.offset + length];
        self.offset += length;
        let raw = raw.strip_suffix(&[0]).unwrap_or(raw);
        std::str::from_utf8(raw)
            .map(str::to_owned)
            .map_err(|error| PakProbeError::Unsupported(format!("invalid UTF-8 FString: {error}")))
    }

    fn require(&self, len: usize) -> Result<(), PakProbeError> {
        if self.offset + len <= self.bytes.len() {
            Ok(())
        } else {
            Err(PakProbeError::Unsupported(format!(
                "pak index ended unexpectedly at byte {} while reading {} bytes",
                self.offset, len
            )))
        }
    }
}
