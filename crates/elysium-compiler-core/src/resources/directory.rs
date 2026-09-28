//! Narrow preflight for ordinary single-volume ZIPs. The ZIP library handles
//! decompression and CRC, but silently folds duplicate directory names. Check the
//! bounded directory first, before that information (or its entry count) is lost.
use anyhow::{ensure, Context, Result};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn short(bytes: &[u8], offset: usize) -> usize {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("fixed header")) as usize
}
fn long(bytes: &[u8], offset: usize) -> u64 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed header")) as u64
}

pub(super) fn check(file: &mut File, length: u64) -> Result<usize> {
    let tail_len = length.min(65535 + 22);
    file.seek(SeekFrom::Start(length - tail_len))?;
    let mut tail = Vec::new();
    file.read_to_end(&mut tail)?;
    let end = tail
        .windows(4)
        .enumerate()
        .rev()
        .find_map(|(i, bytes)| {
            (bytes == b"PK\x05\x06"
                && i + 22 <= tail.len()
                && i + 22 + short(&tail, i + 20) == tail.len())
            .then_some(i)
        })
        .context("missing ZIP end record")?;
    let header = &tail[end..end + 22];
    let count = short(header, 10);
    let size = long(header, 12);
    let start = long(header, 16);
    ensure!(
        short(header, 4) == 0
            && short(header, 6) == 0
            && short(header, 8) == count
            && count <= 60000
            && size <= 32 * 1024 * 1024
            && start + size == length - tail_len + end as u64,
        "unsupported ZIP directory (split, ZIP64, prefixed or over limit)"
    );
    file.seek(SeekFrom::Start(start))?;
    let mut directory = vec![0; size as usize];
    file.read_exact(&mut directory)?;
    let mut names = BTreeSet::new();
    let mut offset = 0;
    for _ in 0..count {
        ensure!(
            offset + 46 <= directory.len() && &directory[offset..offset + 4] == b"PK\x01\x02",
            "invalid ZIP directory record"
        );
        let header = &directory[offset..offset + 46];
        let name_len = short(header, 28);
        let next = offset + 46 + name_len + short(header, 30) + short(header, 32);
        ensure!(
            name_len > 0
                && name_len <= 4096
                && next <= directory.len()
                && short(header, 34) == 0
                && long(header, 42) < start
                && long(header, 20) != u32::MAX as u64
                && long(header, 24) != u32::MAX as u64,
            "invalid or unsupported ZIP entry"
        );
        ensure!(
            names.insert(directory[offset + 46..offset + 46 + name_len].to_vec()),
            "duplicate ZIP entry name"
        );
        offset = next;
    }
    ensure!(
        offset == directory.len(),
        "ZIP entry count does not match directory"
    );
    Ok(count)
}
