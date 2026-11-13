//! The archive: every packaged file, one after another, in a single zstd
//! stream. Inside the stream each file is
//!
//! ```text
//! b'F' | path length (u16 LE) | path (UTF-8, '/'-separated) | size (u64 LE) | bytes
//! ```
//!
//! and a single `b'E'` ends it. One stream compresses better than separate
//! files and can be read front to back without seeking, so extraction
//! needs no temp copy of the archive.

use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::paths;

const FILE: u8 = b'F';
const END: u8 = b'E';

/// How hard the builder compresses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Compression {
    /// Smallest setup file; slowest to build. Unpacking is fast either way.
    #[default]
    Best,
    Normal,
    /// Quickest to build, for trying things out.
    Fast,
}

impl Compression {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "best" => Some(Self::Best),
            "normal" => Some(Self::Normal),
            "fast" => Some(Self::Fast),
            _ => None,
        }
    }

    fn level(self) -> i32 {
        match self {
            Self::Best => 19,
            Self::Normal => 9,
            Self::Fast => 3,
        }
    }
}

/// A file on disk and the path it's packaged under.
#[derive(Clone, Debug)]
pub struct Source {
    pub packaged: String,
    pub path: PathBuf,
    pub size: u64,
}

/// Compresses `sources` into `out`, calling `progress` with the bytes read
/// so far. Returns `out` once the stream is finished.
pub fn write<W: Write>(
    out: W,
    sources: &[Source],
    compression: Compression,
    progress: &mut dyn FnMut(u64),
) -> Result<W> {
    let mut encoder = zstd::stream::Encoder::new(out, compression.level())?;
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()) as u32;
    // Multithreading only changes speed, never the format; ignore a build
    // of zstd without it.
    let _ = encoder.multithread(threads);
    encoder.include_checksum(true)?;

    let mut done = 0u64;
    for source in sources {
        let packaged = paths::normalize(&source.packaged)?;
        let name = packaged.as_bytes();
        let Ok(name_len) = u16::try_from(name.len()) else {
            bail!("\"{packaged}\" is too long a path to package");
        };
        let mut file = File::open(&source.path)
            .with_context(|| format!("couldn't open {}", source.path.display()))?;
        let size = file.metadata()?.len();
        if size != source.size {
            bail!("{} changed size while it was being packaged", source.path.display());
        }
        encoder.write_all(&[FILE])?;
        encoder.write_all(&name_len.to_le_bytes())?;
        encoder.write_all(name)?;
        encoder.write_all(&size.to_le_bytes())?;
        let copied = copy_counting(&mut file, &mut encoder, &mut |n| {
            done = done.saturating_add(n);
            progress(done);
        })
        .with_context(|| format!("couldn't read {}", source.path.display()))?;
        if copied != size {
            bail!("{} changed size while it was being packaged", source.path.display());
        }
    }
    encoder.write_all(&[END])?;
    Ok(encoder.finish()?)
}

/// Reads an archive front to back, handing each file to `visit` with its
/// packaged path, its size and a reader over exactly its bytes. Whatever
/// `visit` leaves unread is skipped.
pub fn read<R: Read>(
    input: R,
    visit: &mut dyn FnMut(&str, u64, &mut dyn Read) -> Result<()>,
) -> Result<()> {
    let mut decoder = zstd::stream::Decoder::new(BufReader::new(input))?;
    loop {
        let mut kind = [0u8; 1];
        decoder.read_exact(&mut kind).context("the archive ends early")?;
        match kind[0] {
            END => return Ok(()),
            FILE => {}
            other => bail!("the archive is damaged (unexpected byte {other:#04x})"),
        }
        let mut len = [0u8; 2];
        decoder.read_exact(&mut len)?;
        let mut name = vec![0u8; usize::from(u16::from_le_bytes(len))];
        decoder.read_exact(&mut name)?;
        let name = String::from_utf8(name).context("the archive holds a path that isn't UTF-8")?;
        let packaged = paths::normalize(&name)?;
        if packaged != name {
            bail!("the archive holds \"{name}\", which isn't a canonical path");
        }
        let mut size = [0u8; 8];
        decoder.read_exact(&mut size)?;
        let size = u64::from_le_bytes(size);

        let mut body = (&mut decoder).take(size);
        visit(&packaged, size, &mut body)?;
        io::copy(&mut body, &mut io::sink())?;
        if body.limit() != 0 {
            bail!("the archive ends in the middle of \"{packaged}\"");
        }
    }
}

/// `io::copy` that reports each chunk as it goes.
pub fn copy_counting(
    from: &mut dyn Read,
    to: &mut dyn Write,
    progress: &mut dyn FnMut(u64),
) -> io::Result<u64> {
    let mut buffer = vec![0u8; 256 * 1024];
    let mut total = 0u64;
    loop {
        let read = match from.read(&mut buffer) {
            Ok(0) => return Ok(total),
            Ok(read) => read,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        };
        to.write_all(&buffer[..read])?;
        total = total.saturating_add(read as u64);
        progress(read as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bach-pack-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn files_round_trip() {
        let dir = temp_dir("archive");
        let big: Vec<u8> = (0..700_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(dir.join("a.bin"), &big).unwrap();
        std::fs::write(dir.join("b.txt"), b"hello").unwrap();
        std::fs::write(dir.join("empty"), b"").unwrap();
        let sources: Vec<Source> = [("bin/a.bin", "a.bin"), ("b.txt", "b.txt"), ("empty", "empty")]
            .iter()
            .map(|(packaged, file)| {
                let path = dir.join(file);
                let size = std::fs::metadata(&path).unwrap().len();
                Source { packaged: packaged.to_string(), path, size }
            })
            .collect();

        let mut last = 0;
        let bytes = write(Vec::new(), &sources, Compression::Fast, &mut |done| last = done).unwrap();
        assert_eq!(last, big.len() as u64 + 5);
        assert!(bytes.len() < big.len() / 10, "the pattern should compress");

        let mut seen = Vec::new();
        read(bytes.as_slice(), &mut |path, size, body| {
            let mut data = Vec::new();
            body.read_to_end(&mut data)?;
            assert_eq!(data.len() as u64, size);
            seen.push((path.to_string(), data));
            Ok(())
        })
        .unwrap();
        assert_eq!(seen.len(), 3);
        assert_eq!(seen[0].0, "bin/a.bin");
        assert_eq!(seen[0].1, big);
        assert_eq!(seen[1].1, b"hello");
        assert!(seen[2].1.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unread_bodies_are_skipped() {
        let dir = temp_dir("skip");
        std::fs::write(dir.join("one"), b"first file").unwrap();
        std::fs::write(dir.join("two"), b"second").unwrap();
        let sources: Vec<Source> = ["one", "two"]
            .iter()
            .map(|name| Source { packaged: name.to_string(), path: dir.join(name), size: std::fs::metadata(dir.join(name)).unwrap().len() })
            .collect();
        let bytes = write(Vec::new(), &sources, Compression::Fast, &mut |_| {}).unwrap();
        let mut names = Vec::new();
        read(bytes.as_slice(), &mut |path, _, _| {
            names.push(path.to_string());
            Ok(())
        })
        .unwrap();
        assert_eq!(names, ["one", "two"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn truncated_archives_fail() {
        let dir = temp_dir("truncated");
        std::fs::write(dir.join("one"), vec![1u8; 4096]).unwrap();
        let sources = vec![Source { packaged: "one".into(), path: dir.join("one"), size: 4096 }];
        let bytes = write(Vec::new(), &sources, Compression::Fast, &mut |_| {}).unwrap();
        let cut = &bytes[..bytes.len() / 2];
        assert!(read(cut, &mut |_, _, _| Ok(())).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
