//! How a payload rides on the setup runtime's executable.
//!
//! A setup file is the `bach-setup.exe` runtime with three sections and a
//! fixed-size trailer appended after its last byte:
//!
//! ```text
//! [runtime exe] [archive] [logo] [manifest JSON] [padding] [trailer]
//! trailer = "BACHPACK" | format u32 | padding u32 | archive len u64 | logo len u64 | manifest len u64
//! ```
//!
//! Windows ignores bytes after an executable's last section, so the file
//! still runs as the runtime, which then finds its payload from the end.
//! The padding keeps the file a multiple of 8 bytes long, so `signtool`
//! adds no padding of its own when it signs it. A signature goes after the
//! trailer, inside the PE's certificate table; [`payload_end`] reads the
//! table's offset so signed setup files keep working, and the signature
//! covers the payload too.
//!
//! The uninstaller is the same runtime with a manifest and no archive.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use anyhow::{bail, Context, Result};

use crate::manifest::Manifest;

pub const MAGIC: &[u8; 8] = b"BACHPACK";
pub const TRAILER_LEN: u64 = 40;
/// Files are padded to this, matching Authenticode's alignment.
const ALIGN: u64 = 8;
/// A manifest this large is damage, not a real app.
const MAX_MANIFEST: u64 = 64 << 20;
const MAX_LOGO: u64 = 16 << 20;

/// A byte range in the setup file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Section {
    pub offset: u64,
    pub len: u64,
}

/// Where everything sits in a setup file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    /// The runtime executable is the bytes before this.
    pub runtime_len: u64,
    pub archive: Section,
    pub logo: Section,
    pub manifest: Section,
}

impl Layout {
    /// An uninstaller carries no files.
    pub fn has_archive(&self) -> bool {
        self.archive.len > 0
    }
}

/// Appends the payload to a file that already holds the runtime, with the
/// archive already written right after it, starting at `archive_offset`.
/// Writes the logo, the manifest, padding and the trailer.
pub fn finish(file: &mut File, archive_offset: u64, logo: &[u8], manifest: &Manifest) -> Result<Layout> {
    let archive_end = file.seek(SeekFrom::End(0))?;
    let Some(archive_len) = archive_end.checked_sub(archive_offset) else {
        bail!("the archive starts after the end of the file");
    };
    file.write_all(logo)?;
    let json = serde_json::to_vec(manifest)?;
    file.write_all(&json)?;
    let end = file.stream_position()?;
    let padding = (ALIGN - (end + TRAILER_LEN) % ALIGN) % ALIGN;
    file.write_all(&vec![0u8; padding as usize])?;

    let trailer = Trailer {
        padding: padding as u32,
        archive_len,
        logo_len: logo.len() as u64,
        manifest_len: json.len() as u64,
    };
    file.write_all(&trailer.encode())?;
    file.flush()?;
    Ok(Layout {
        runtime_len: archive_offset,
        archive: Section { offset: archive_offset, len: archive_len },
        logo: Section { offset: archive_end, len: trailer.logo_len },
        manifest: Section { offset: archive_end + trailer.logo_len, len: trailer.manifest_len },
    })
}

/// Finds the payload in a setup file, or None when there is none (the bare
/// runtime).
pub fn locate(file: &mut File) -> Result<Option<Layout>> {
    let end = payload_end(file)?;
    // signtool pads to 8 bytes before the certificate table; our own
    // padding normally makes that a no-op, but look past a few zeros anyway.
    for skip in 0..ALIGN {
        let Some(trailer_at) = end.checked_sub(TRAILER_LEN + skip) else {
            return Ok(None);
        };
        let mut raw = [0u8; TRAILER_LEN as usize];
        file.seek(SeekFrom::Start(trailer_at))?;
        file.read_exact(&mut raw)?;
        if let Some(trailer) = Trailer::decode(&raw)? {
            return trailer.layout(trailer_at).map(Some);
        }
    }
    Ok(None)
}

/// Reads and checks the manifest.
pub fn read_manifest(file: &mut File, layout: &Layout) -> Result<Manifest> {
    let bytes = read_section(file, layout.manifest, MAX_MANIFEST)?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).context("the setup file's manifest is damaged")?;
    manifest.validate()?;
    Ok(manifest)
}

/// Reads the logo's bytes (empty when there is none).
pub fn read_logo(file: &mut File, layout: &Layout) -> Result<Vec<u8>> {
    read_section(file, layout.logo, MAX_LOGO)
}

fn read_section(file: &mut File, section: Section, max: u64) -> Result<Vec<u8>> {
    if section.len > max {
        bail!("the setup file is damaged (a {} byte section)", section.len);
    }
    file.seek(SeekFrom::Start(section.offset))?;
    let mut bytes = vec![0u8; section.len as usize];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// Writes an uninstaller: the runtime's bytes, without any signature, then
/// the logo and the manifest with no archive.
pub fn write_uninstaller(out: &mut File, runtime: &[u8], logo: &[u8], manifest: &Manifest) -> Result<()> {
    let mut runtime = runtime.to_vec();
    // The setup file's signature doesn't cover this new file; a certificate
    // table pointing past its end would only make Windows distrust it.
    clear_certificate_table(&mut runtime);
    out.write_all(&runtime)?;
    let offset = runtime.len() as u64;
    finish(out, offset, logo, manifest)?;
    Ok(())
}

struct Trailer {
    /// Zero bytes between the manifest and the trailer.
    padding: u32,
    archive_len: u64,
    logo_len: u64,
    manifest_len: u64,
}

impl Trailer {
    fn encode(&self) -> [u8; TRAILER_LEN as usize] {
        let mut raw = [0u8; TRAILER_LEN as usize];
        raw[0..8].copy_from_slice(MAGIC);
        raw[8..12].copy_from_slice(&crate::FORMAT_VERSION.to_le_bytes());
        raw[12..16].copy_from_slice(&self.padding.to_le_bytes());
        raw[16..24].copy_from_slice(&self.archive_len.to_le_bytes());
        raw[24..32].copy_from_slice(&self.logo_len.to_le_bytes());
        raw[32..40].copy_from_slice(&self.manifest_len.to_le_bytes());
        raw
    }

    /// None when the bytes aren't a trailer at all.
    fn decode(raw: &[u8; TRAILER_LEN as usize]) -> Result<Option<Self>> {
        if &raw[0..8] != MAGIC {
            return Ok(None);
        }
        let format = u32::from_le_bytes(word(raw, 8));
        if format != crate::FORMAT_VERSION {
            bail!(
                "this setup file uses payload format {format}, but this runtime reads format {}. \
                 Build it again with the matching Bach",
                crate::FORMAT_VERSION
            );
        }
        Ok(Some(Self {
            padding: u32::from_le_bytes(word(raw, 12)),
            archive_len: u64::from_le_bytes(dword(raw, 16)),
            logo_len: u64::from_le_bytes(dword(raw, 24)),
            manifest_len: u64::from_le_bytes(dword(raw, 32)),
        }))
    }

    /// Works back from where the trailer starts.
    fn layout(&self, trailer_at: u64) -> Result<Layout> {
        let damaged = || anyhow::anyhow!("the setup file is damaged (its sections don't fit)");
        let runtime_len = trailer_at
            .checked_sub(u64::from(self.padding))
            .and_then(|end| end.checked_sub(self.manifest_len))
            .and_then(|end| end.checked_sub(self.logo_len))
            .and_then(|end| end.checked_sub(self.archive_len))
            .ok_or_else(damaged)?;
        let archive = Section { offset: runtime_len, len: self.archive_len };
        let logo = Section { offset: archive.offset + archive.len, len: self.logo_len };
        let manifest = Section { offset: logo.offset + logo.len, len: self.manifest_len };
        Ok(Layout { runtime_len, archive, logo, manifest })
    }
}

fn word(raw: &[u8], at: usize) -> [u8; 4] {
    let mut out = [0u8; 4];
    out.copy_from_slice(&raw[at..at + 4]);
    out
}

fn dword(raw: &[u8], at: usize) -> [u8; 8] {
    let mut out = [0u8; 8];
    out.copy_from_slice(&raw[at..at + 8]);
    out
}

/// Where the payload ends: the start of the PE certificate table when the
/// file is signed, otherwise the end of the file.
pub fn payload_end(file: &mut File) -> Result<u64> {
    let len = file.seek(SeekFrom::End(0))?;
    let mut head = vec![0u8; 4096.min(len as usize)];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut head)?;
    Ok(match certificate_table(&head) {
        Some((offset, size)) if size > 0 && offset > 0 && offset <= len => offset,
        _ => len,
    })
}

/// The position in the PE header of the certificate table's data directory
/// entry (offset and size, 4 bytes each), if the header has one.
fn certificate_entry(head: &[u8]) -> Option<usize> {
    if head.get(0..2)? != b"MZ" {
        return None;
    }
    let pe = u32::from_le_bytes(head.get(0x3c..0x40)?.try_into().ok()?) as usize;
    if head.get(pe..pe + 4)? != b"PE\0\0" {
        return None;
    }
    let optional = pe + 24;
    let magic = u16::from_le_bytes(head.get(optional..optional + 2)?.try_into().ok()?);
    let (count_at, dirs_at) = match magic {
        0x10b => (optional + 92, optional + 96),
        0x20b => (optional + 108, optional + 112),
        _ => return None,
    };
    let count = u32::from_le_bytes(head.get(count_at..count_at + 4)?.try_into().ok()?);
    // The certificate table is data directory 4.
    if count < 5 {
        return None;
    }
    let entry = dirs_at + 4 * 8;
    head.get(entry..entry + 8)?;
    Some(entry)
}

fn certificate_table(head: &[u8]) -> Option<(u64, u64)> {
    let entry = certificate_entry(head)?;
    let offset = u32::from_le_bytes(head.get(entry..entry + 4)?.try_into().ok()?);
    let size = u32::from_le_bytes(head.get(entry + 4..entry + 8)?.try_into().ok()?);
    Some((u64::from(offset), u64::from(size)))
}

/// Zeroes the certificate table entry, unsigning a copy of the runtime.
pub fn clear_certificate_table(exe: &mut [u8]) {
    let head_len = exe.len().min(4096);
    if let Some(entry) = certificate_entry(&exe[..head_len]) {
        if let Some(bytes) = exe.get_mut(entry..entry + 8) {
            bytes.fill(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::sample;

    fn temp_file(name: &str) -> (std::path::PathBuf, File) {
        let path = std::env::temp_dir().join(format!("bach-container-{name}-{}", std::process::id()));
        let file = File::options().read(true).write(true).create(true).truncate(true).open(&path).unwrap();
        (path, file)
    }

    /// A minimal PE32+ header, optionally with a certificate table entry.
    fn fake_pe(cert: Option<(u32, u32)>) -> Vec<u8> {
        let mut exe = vec![0u8; 512];
        exe[0..2].copy_from_slice(b"MZ");
        exe[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        exe[0x80..0x84].copy_from_slice(b"PE\0\0");
        let optional = 0x80 + 24;
        exe[optional..optional + 2].copy_from_slice(&0x20bu16.to_le_bytes());
        exe[optional + 108..optional + 112].copy_from_slice(&16u32.to_le_bytes());
        if let Some((offset, size)) = cert {
            let entry = optional + 112 + 32;
            exe[entry..entry + 4].copy_from_slice(&offset.to_le_bytes());
            exe[entry + 4..entry + 8].copy_from_slice(&size.to_le_bytes());
        }
        exe
    }

    fn build(file: &mut File, runtime: &[u8], archive: &[u8], logo: &[u8]) -> Layout {
        file.write_all(runtime).unwrap();
        file.write_all(archive).unwrap();
        finish(file, runtime.len() as u64, logo, &sample()).unwrap()
    }

    #[test]
    fn payloads_are_found_again() {
        let (path, mut file) = temp_file("plain");
        let runtime = fake_pe(None);
        let written = build(&mut file, &runtime, b"ARCHIVE", b"LOGO!");
        assert_eq!(file.seek(SeekFrom::End(0)).unwrap() % ALIGN, 0);

        let found = locate(&mut file).unwrap().unwrap();
        assert_eq!(found, written);
        assert_eq!(found.runtime_len, runtime.len() as u64);
        assert!(found.has_archive());
        assert_eq!(read_manifest(&mut file, &found).unwrap(), sample());
        assert_eq!(read_logo(&mut file, &found).unwrap(), b"LOGO!");
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn a_bare_runtime_has_no_payload() {
        let (path, mut file) = temp_file("bare");
        file.write_all(&fake_pe(None)).unwrap();
        assert!(locate(&mut file).unwrap().is_none());
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn signed_files_keep_their_payload() {
        let (path, mut file) = temp_file("signed");
        let runtime = fake_pe(None);
        let written = build(&mut file, &runtime, b"ARCHIVE", b"");
        // Sign it: a certificate table after the payload, recorded in the header.
        let cert_at = file.seek(SeekFrom::End(0)).unwrap();
        file.write_all(&[0xAB; 64]).unwrap();
        let signed = fake_pe(Some((cert_at as u32, 64)));
        file.seek(SeekFrom::Start(0)).unwrap();
        file.write_all(&signed).unwrap();

        let found = locate(&mut file).unwrap().unwrap();
        assert_eq!(found, written);
        assert_eq!(read_manifest(&mut file, &found).unwrap(), sample());
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn uninstallers_carry_no_archive_and_no_signature() {
        let (path, mut file) = temp_file("uninstaller");
        let signed = fake_pe(Some((4096, 64)));
        write_uninstaller(&mut file, &signed, b"", &sample()).unwrap();
        let found = locate(&mut file).unwrap().unwrap();
        assert!(!found.has_archive());
        assert_eq!(read_manifest(&mut file, &found).unwrap(), sample());

        let mut head = vec![0u8; 512];
        file.seek(SeekFrom::Start(0)).unwrap();
        file.read_exact(&mut head).unwrap();
        assert_eq!(certificate_table(&head), Some((0, 0)));
        drop(file);
        std::fs::remove_file(path).unwrap();
    }
}
