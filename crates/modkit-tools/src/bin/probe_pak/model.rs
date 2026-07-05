#[derive(Debug)]
struct PakProbe {
    pak_path: PathBuf,
    file_size: u64,
    footer: PakFooter,
    index_prefix_seed: Option<u32>,
    mount_point: String,
    entries: Vec<PakEntry>,
}

#[derive(Debug)]
struct PakFooter {
    footer_offset: u64,
    version: i32,
    index_offset: u64,
    index_size: u64,
    index_hash: [u8; 20],
}

#[derive(Debug, Eq, PartialEq)]
struct PakEntry {
    path: String,
    offset: u64,
    size: u64,
    uncompressed_size: u64,
    compression_method: u32,
    hash: [u8; 20],
    encrypted: bool,
    compression_block_size: u32,
}

#[derive(Debug)]
struct ExtractedEntry {
    path: String,
    output_path: PathBuf,
    offset: u64,
    data_offset: u64,
    size: u64,
    sha1: [u8; 20],
}
