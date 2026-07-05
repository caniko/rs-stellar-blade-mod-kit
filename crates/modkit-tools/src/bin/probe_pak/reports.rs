fn write_summary(path: &Path, probe: &PakProbe) -> Result<(), PakProbeError> {
    let mut writer = writer(path)?;
    writeln!(writer, "# Pak Probe Summary")?;
    writeln!(writer)?;
    writeln!(writer, "- Pak: `{}`", probe.pak_path.display())?;
    writeln!(writer, "- File size: {}", probe.file_size)?;
    writeln!(writer, "- Footer offset: {}", probe.footer.footer_offset)?;
    writeln!(writer, "- Pak version: {}", probe.footer.version)?;
    writeln!(writer, "- Index offset: {}", probe.footer.index_offset)?;
    writeln!(writer, "- Index size: {}", probe.footer.index_size)?;
    writeln!(writer, "- Index hash: `{}`", hex(&probe.footer.index_hash))?;
    writeln!(
        writer,
        "- Index prefix seed: `{}`",
        probe
            .index_prefix_seed
            .map_or_else(|| "none".to_owned(), |seed| format!("0x{seed:08x}"))
    )?;
    writeln!(writer, "- Mount point: `{}`", probe.mount_point)?;
    writeln!(writer, "- Entries: {}", probe.entries.len())?;
    writeln!(writer)?;
    writeln!(writer, "## Entries")?;
    for entry in &probe.entries {
        writeln!(
            writer,
            "- `{}` offset={} size={} uncompressed={} encrypted={} sha1=`{}`",
            entry.path,
            entry.offset,
            entry.size,
            entry.uncompressed_size,
            entry.encrypted,
            hex(&entry.hash)
        )?;
    }
    Ok(())
}

fn write_entries_jsonl(path: &Path, probe: &PakProbe) -> Result<(), PakProbeError> {
    let mut writer = writer(path)?;
    for entry in &probe.entries {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"pak_path\":{},\"mount_point\":{},\"path\":{},\"offset\":{},\"size\":{},\"uncompressed_size\":{},\"compression_method\":{},\"encrypted\":{},\"compression_block_size\":{},\"sha1\":{}}}",
            json_string(&probe.pak_path.to_string_lossy()),
            json_string(&probe.mount_point),
            json_string(&entry.path),
            entry.offset,
            entry.size,
            entry.uncompressed_size,
            entry.compression_method,
            entry.encrypted,
            entry.compression_block_size,
            json_string(&hex(&entry.hash))
        )?;
    }
    Ok(())
}
