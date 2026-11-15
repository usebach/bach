//! Keeping a setup file's contents from anyone without the access key.
//!
//! The key is turned into an encryption key with Argon2id (slow on purpose,
//! so guessing keys is expensive), and the archive is encrypted in 64 KiB
//! chunks with ChaCha20-Poly1305 in the STREAM construction: every chunk is
//! authenticated, and the last one is marked, so a flipped bit, a chunk cut
//! out of the middle and a file cut short all fail loudly instead of
//! unpacking something wrong.
//!
//! What this protects: the packaged files, at rest in the setup file. What
//! it does not: the file names and sizes, which the manifest lists in the
//! clear, and anything after install, because an installed app's files are
//! ordinary files on the user's disk.
//!
//! The scheme is fixed ([`SCHEME`]); a manifest that names another one is
//! refused rather than guessed at.

use std::io::{self, Read, Write};

use anyhow::{bail, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::stream::{DecryptorBE32, EncryptorBE32};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use sha2::{Digest, Sha256};

use crate::manifest::Encryption;

/// The one scheme this version writes and reads.
pub const SCHEME: &str = "argon2id-chacha20poly1305-v1";
/// Plain bytes per encrypted chunk.
pub const CHUNK: usize = 64 * 1024;
const TAG: usize = 16;
/// Bytes of the STREAM nonce prefix (a 12 byte nonce minus a counter and a flag).
pub const NONCE_LEN: usize = 7;
const SALT_LEN: usize = 16;
/// What new setup files use: 64 MiB and 3 passes, about a third of a second.
pub const MEMORY_KIB: u32 = 64 * 1024;
pub const PASSES: u32 = 3;
/// A setup file asking for more than this is not one Bach wrote: refusing
/// it keeps a hostile file from exhausting the computer.
pub const MAX_MEMORY_KIB: u32 = 256 * 1024;
pub const MAX_PASSES: u32 = 10;
/// The shortest and longest key a build accepts.
pub const MIN_KEY: usize = 8;
pub const MAX_KEY: usize = 128;

/// What a key unlocks. Never printed; wiped when dropped.
pub struct Unlocked {
    key: [u8; 32],
}

impl std::fmt::Debug for Unlocked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Unlocked(..)")
    }
}

impl Drop for Unlocked {
    fn drop(&mut self) {
        // Volatile, so the compiler can't drop the write as dead.
        for byte in &mut self.key {
            // Safety: `byte` is a valid, aligned reference into `self.key`.
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
    }
}

/// Why a key didn't open a setup file.
#[derive(Debug, PartialEq, Eq)]
pub enum KeyError {
    /// The key is not the one the files were locked with.
    Wrong,
    /// The setup file's own protection data is unusable.
    Damaged(String),
}

impl std::fmt::Display for KeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wrong => f.write_str("that key doesn't open this setup file"),
            Self::Damaged(why) => write!(f, "the setup file's protection is damaged ({why})"),
        }
    }
}

impl std::error::Error for KeyError {}

/// A key a build can use: `Err` says why not, in a sentence that fits after
/// "the access key".
pub fn check_key(key: &str) -> Result<(), String> {
    if key.len() < MIN_KEY {
        return Err(format!("is too short; use at least {MIN_KEY} characters"));
    }
    if key.len() > MAX_KEY {
        return Err(format!("is too long; use at most {MAX_KEY} characters"));
    }
    if let Some(bad) = key.chars().find(|c| !c.is_ascii_alphanumeric() && !"-_.~+/=:".contains(*c)) {
        return Err(format!(
            "has {bad:?} in it; use letters, digits and - _ . ~ + / = : (it travels in an HTTP header too)"
        ));
    }
    Ok(())
}

/// A new random key, like `K7QF-3MWD-9XPA-H2TY-6NBC`: 100 bits, no letters
/// that look alike (no 0 O 1 I).
pub fn generate_key() -> Result<String> {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut random = [0u8; 20];
    fill_random(&mut random)?;
    let mut key = String::with_capacity(24);
    for (index, byte) in random.iter().enumerate() {
        if index > 0 && index % 4 == 0 {
            key.push('-');
        }
        // 32 symbols and 256 byte values: every symbol is equally likely.
        key.push(ALPHABET[usize::from(byte & 31)] as char);
    }
    Ok(key)
}

fn fill_random(bytes: &mut [u8]) -> Result<()> {
    getrandom::fill(bytes).map_err(|err| anyhow::anyhow!("couldn't get random bytes from the system: {err}"))
}

impl Encryption {
    /// Protection for a new setup file, with a fresh salt and nonce, and
    /// what `key` unlocks.
    pub fn create(key: &str) -> Result<(Self, Unlocked)> {
        if let Err(why) = check_key(key) {
            bail!("the access key {why}");
        }
        let mut salt = [0u8; SALT_LEN];
        let mut nonce = [0u8; NONCE_LEN];
        fill_random(&mut salt)?;
        fill_random(&mut nonce)?;
        let unlocked = derive(key, &salt, MEMORY_KIB, PASSES).map_err(|why| anyhow::anyhow!("{why}"))?;
        let encryption = Self {
            scheme: SCHEME.into(),
            salt: hex(&salt),
            nonce: hex(&nonce),
            memory_kib: MEMORY_KIB,
            passes: PASSES,
            check: check_value(&unlocked),
        };
        Ok((encryption, unlocked))
    }

    /// Whether these settings are ones this version can use, without
    /// deriving anything. A setup file is untrusted input.
    pub fn validate(&self) -> Result<()> {
        if self.scheme != SCHEME {
            bail!("this setup file is protected in a way (\"{}\") this version of setup doesn't know", self.scheme);
        }
        if unhex::<SALT_LEN>(&self.salt).is_none() || unhex::<NONCE_LEN>(&self.nonce).is_none() {
            bail!("the setup file's protection data is damaged");
        }
        if self.check.len() != 64 || !self.check.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("the setup file's protection data is damaged");
        }
        if !(8..=MAX_MEMORY_KIB).contains(&self.memory_kib) || !(1..=MAX_PASSES).contains(&self.passes) {
            bail!("the setup file asks for more memory or time than setup allows to check a key");
        }
        Ok(())
    }

    /// Opens with `key`: slow (that is the point), so call it off the UI
    /// thread.
    pub fn unlock(&self, key: &str) -> Result<Unlocked, KeyError> {
        self.validate().map_err(|err| KeyError::Damaged(err.to_string()))?;
        let salt = unhex::<SALT_LEN>(&self.salt).ok_or_else(|| KeyError::Damaged("salt".into()))?;
        // A key the build would never have accepted can't be the right one.
        if check_key(key).is_err() {
            return Err(KeyError::Wrong);
        }
        let unlocked = derive(key, &salt, self.memory_kib, self.passes).map_err(KeyError::Damaged)?;
        if same(check_value(&unlocked).as_bytes(), self.check.to_ascii_lowercase().as_bytes()) {
            Ok(unlocked)
        } else {
            Err(KeyError::Wrong)
        }
    }

    /// The STREAM nonce prefix the chunks were sealed with.
    pub fn nonce(&self) -> Result<[u8; NONCE_LEN]> {
        unhex::<NONCE_LEN>(&self.nonce).ok_or_else(|| anyhow::anyhow!("the setup file's protection data is damaged"))
    }

    /// Encrypts everything written to `out`; call [`EncryptWriter::finish`].
    pub fn writer<W: Write>(&self, out: W, unlocked: &Unlocked) -> Result<EncryptWriter<W>> {
        Ok(EncryptWriter::new(out, unlocked, &self.nonce()?))
    }

    /// Decrypts what is read from `input`.
    pub fn reader<R: Read>(&self, input: R, unlocked: &Unlocked) -> Result<DecryptReader<R>> {
        Ok(DecryptReader::new(input, unlocked, &self.nonce()?))
    }
}

fn derive(key: &str, salt: &[u8], memory_kib: u32, passes: u32) -> Result<Unlocked, String> {
    let params = Params::new(memory_kib, passes, 1, Some(32)).map_err(|err| err.to_string())?;
    let mut out = [0u8; 32];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(key.as_bytes(), salt, &mut out)
        .map_err(|err| err.to_string())?;
    Ok(Unlocked { key: out })
}

/// What the manifest keeps to tell a right key from a wrong one: a hash of
/// the derived key under a label, so it reveals nothing the chunks don't.
fn check_value(unlocked: &Unlocked) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"bach access check v1");
    hasher.update(unlocked.key);
    hex(&hasher.finalize())
}

/// Equal in time that doesn't depend on where they differ.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// SHA-256 of everything `reader` yields, as lowercase hex. sha2 0.11
/// dropped its `io::Write` impl, so `io::copy` into a hasher no longer works.
pub fn sha256_hex(mut reader: impl Read) -> std::io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buffer[..n]),
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
            Err(err) => return Err(err),
        }
    }
    Ok(hex(&hasher.finalize()))
}

/// Lowercase hex text for `bytes` (what a SHA-256 digest is shown as; the
/// digest type itself no longer formats as hex since sha2 0.11).
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex<const N: usize>(text: &str) -> Option<[u8; N]> {
    if text.len() != N * 2 || !text.is_ascii() {
        return None;
    }
    let mut out = [0u8; N];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(out)
}

type Sealer = EncryptorBE32<ChaCha20Poly1305>;
type Opener = DecryptorBE32<ChaCha20Poly1305>;

fn cipher(unlocked: &Unlocked) -> ChaCha20Poly1305 {
    ChaCha20Poly1305::new(GenericArray::from_slice(&unlocked.key))
}

/// A writer that encrypts what it is given, chunk by chunk, into `inner`.
pub struct EncryptWriter<W: Write> {
    inner: W,
    sealer: Option<Sealer>,
    buffer: Vec<u8>,
}

impl<W: Write> EncryptWriter<W> {
    pub fn new(inner: W, unlocked: &Unlocked, nonce: &[u8; NONCE_LEN]) -> Self {
        let sealer = Sealer::from_aead(cipher(unlocked), GenericArray::from_slice(nonce));
        Self { inner, sealer: Some(sealer), buffer: Vec::with_capacity(CHUNK * 2) }
    }

    /// Seals what is left as the last chunk and gives `inner` back.
    pub fn finish(mut self) -> io::Result<W> {
        let sealer = self.sealer.take().ok_or_else(|| io::Error::other("already finished"))?;
        let last = sealer.encrypt_last(self.buffer.as_slice()).map_err(|_| io::Error::other("couldn't seal the last chunk"))?;
        self.inner.write_all(&last)?;
        self.inner.flush()?;
        Ok(self.inner)
    }
}

impl<W: Write> Write for EncryptWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(bytes);
        // Strictly more than a chunk: the last chunk is never empty.
        while self.buffer.len() > CHUNK {
            let sealer = self.sealer.as_mut().ok_or_else(|| io::Error::other("already finished"))?;
            let sealed = sealer
                .encrypt_next(&self.buffer[..CHUNK])
                .map_err(|_| io::Error::other("couldn't seal a chunk"))?;
            self.buffer.drain(..CHUNK);
            self.inner.write_all(&sealed)?;
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Buffered bytes stay buffered: a chunk is sealed whole.
        self.inner.flush()
    }
}

/// A reader that decrypts what `inner` holds. It looks one chunk ahead, to
/// know which chunk is the last; a file that stops before its last chunk, or
/// any chunk that doesn't check out, is an `InvalidData` error.
pub struct DecryptReader<R: Read> {
    inner: R,
    opener: Option<Opener>,
    /// The next sealed chunk, already read.
    ahead: Option<Vec<u8>>,
    started: bool,
    plain: Vec<u8>,
    taken: usize,
    finished: bool,
}

impl<R: Read> DecryptReader<R> {
    pub fn new(inner: R, unlocked: &Unlocked, nonce: &[u8; NONCE_LEN]) -> Self {
        let opener = Opener::from_aead(cipher(unlocked), GenericArray::from_slice(nonce));
        Self { inner, opener: Some(opener), ahead: None, started: false, plain: Vec::new(), taken: 0, finished: false }
    }

    /// Up to one sealed chunk; shorter only at the end of the input.
    fn read_chunk(&mut self) -> io::Result<Vec<u8>> {
        let mut chunk = vec![0u8; CHUNK + TAG];
        let mut filled = 0;
        while filled < chunk.len() {
            match self.inner.read(&mut chunk[filled..]) {
                Ok(0) => break,
                Ok(read) => filled += read,
                Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
                Err(err) => return Err(err),
            }
        }
        chunk.truncate(filled);
        Ok(chunk)
    }

    fn open_next(&mut self) -> io::Result<()> {
        let damaged = || io::Error::new(io::ErrorKind::InvalidData, "the protected data is damaged or cut short");
        let chunk = match self.ahead.take() {
            Some(chunk) => chunk,
            None if !self.started => self.read_chunk()?,
            None => return Err(damaged()),
        };
        self.started = true;
        // A full chunk may or may not be the last: read on to find out.
        let following = if chunk.len() == CHUNK + TAG { self.read_chunk()? } else { Vec::new() };
        let last = following.is_empty();
        let opener = self.opener.as_mut();
        self.plain = if last {
            self.opener.take().ok_or_else(damaged)?.decrypt_last(chunk.as_slice()).map_err(|_| damaged())?
        } else {
            opener.ok_or_else(damaged)?.decrypt_next(chunk.as_slice()).map_err(|_| damaged())?
        };
        self.taken = 0;
        if last {
            self.finished = true;
        } else {
            self.ahead = Some(following);
        }
        Ok(())
    }
}

impl<R: Read> Read for DecryptReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        while self.taken == self.plain.len() {
            if self.finished {
                return Ok(0);
            }
            self.open_next()?;
        }
        let count = out.len().min(self.plain.len() - self.taken);
        out[..count].copy_from_slice(&self.plain[self.taken..self.taken + count]);
        self.taken += count;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn protection() -> (Encryption, Unlocked) {
        // Cheap settings keep the tests quick; the format is the same.
        let salt = [7u8; SALT_LEN];
        let nonce = [9u8; NONCE_LEN];
        let unlocked = derive("correct-horse-battery", &salt, 8, 1).unwrap();
        let encryption = Encryption {
            scheme: SCHEME.into(),
            salt: hex(&salt),
            nonce: hex(&nonce),
            memory_kib: 8,
            passes: 1,
            check: check_value(&unlocked),
        };
        (encryption, unlocked)
    }

    fn seal(plain: &[u8]) -> Vec<u8> {
        let (encryption, unlocked) = protection();
        let mut writer = encryption.writer(Vec::new(), &unlocked).unwrap();
        // Uneven writes, so chunk edges fall in the middle of calls.
        for piece in plain.chunks(10_007) {
            writer.write_all(piece).unwrap();
        }
        writer.finish().unwrap()
    }

    fn open(sealed: &[u8]) -> io::Result<Vec<u8>> {
        let (encryption, unlocked) = protection();
        let mut out = Vec::new();
        encryption.reader(sealed, &unlocked).unwrap().read_to_end(&mut out).map(|_| out)
    }

    fn pattern(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i.wrapping_mul(31) ^ (i >> 7)) as u8).collect()
    }

    #[test]
    fn data_round_trips_at_every_awkward_length() {
        for len in [0, 1, 15, CHUNK - 1, CHUNK, CHUNK + 1, 2 * CHUNK, 2 * CHUNK + 1, 3 * CHUNK - 1, 300_000] {
            let plain = pattern(len);
            let sealed = seal(&plain);
            assert!(sealed.len() >= len + TAG, "{len}");
            assert_eq!(open(&sealed).unwrap(), plain, "length {len}");
        }
    }

    #[test]
    fn sealed_data_doesnt_contain_the_plain_data() {
        let plain = b"a very recognisable sentence that must not show".repeat(100);
        let sealed = seal(&plain);
        assert!(!sealed.windows(24).any(|window| window == &plain[..24]));
    }

    #[test]
    fn a_flipped_bit_is_caught() {
        let plain = pattern(3 * CHUNK + 5);
        let sealed = seal(&plain);
        for at in [0, 100, CHUNK + TAG + 3, sealed.len() - 1] {
            let mut damaged = sealed.clone();
            damaged[at] ^= 1;
            let err = open(&damaged).unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidData, "bit at {at}");
        }
    }

    #[test]
    fn a_file_cut_short_is_caught_even_at_a_chunk_edge() {
        let plain = pattern(3 * CHUNK + 5);
        let sealed = seal(&plain);
        let chunk = CHUNK + TAG;
        for cut in [0, 10, chunk, 2 * chunk, 3 * chunk, sealed.len() - 1] {
            let err = open(&sealed[..cut]).unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidData, "cut at {cut}");
        }
    }

    #[test]
    fn chunks_cant_be_reordered_or_dropped() {
        let plain = pattern(3 * CHUNK);
        let sealed = seal(&plain);
        let chunk = CHUNK + TAG;
        let mut swapped = Vec::new();
        swapped.extend_from_slice(&sealed[chunk..2 * chunk]);
        swapped.extend_from_slice(&sealed[..chunk]);
        swapped.extend_from_slice(&sealed[2 * chunk..]);
        assert!(open(&swapped).is_err());
        let mut dropped = sealed[..chunk].to_vec();
        dropped.extend_from_slice(&sealed[2 * chunk..]);
        assert!(open(&dropped).is_err());
    }

    #[test]
    fn extra_bytes_after_the_end_are_caught() {
        let mut sealed = seal(&pattern(1000));
        sealed.extend_from_slice(&[0u8; 40]);
        assert!(open(&sealed).is_err());
    }

    #[test]
    fn a_wrong_key_is_told_apart_before_anything_is_decrypted() {
        let (encryption, _) = protection();
        assert!(encryption.unlock("correct-horse-battery").is_ok());
        assert_eq!(encryption.unlock("correct-horse-batterz").unwrap_err(), KeyError::Wrong);
        assert_eq!(encryption.unlock("").unwrap_err(), KeyError::Wrong);
        assert_eq!(encryption.unlock("has spaces in it").unwrap_err(), KeyError::Wrong);
    }

    #[test]
    fn the_wrong_key_cant_open_the_data_either() {
        let sealed = seal(&pattern(5000));
        let (encryption, _) = protection();
        let other = derive("another-key-entirely", &unhex::<SALT_LEN>(&encryption.salt).unwrap(), 8, 1).unwrap();
        let mut reader = encryption.reader(sealed.as_slice(), &other).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
    }

    #[test]
    fn created_protection_unlocks_with_its_key_only() {
        let (encryption, unlocked) = Encryption::create("K7QF-3MWD-9XPA-H2TY-6NBC").unwrap();
        encryption.validate().unwrap();
        let again = encryption.unlock("K7QF-3MWD-9XPA-H2TY-6NBC").unwrap();
        assert_eq!(again.key, unlocked.key);
        assert_eq!(encryption.unlock("K7QF-3MWD-9XPA-H2TY-6NBD").unwrap_err(), KeyError::Wrong);
        // Fresh salt and nonce every time: two builds never share a key stream.
        let (other, _) = Encryption::create("K7QF-3MWD-9XPA-H2TY-6NBC").unwrap();
        assert_ne!(encryption.salt, other.salt);
        assert_ne!(encryption.nonce, other.nonce);
        assert_ne!(encryption.check, other.check);
    }

    #[test]
    fn hostile_protection_data_is_refused_not_obeyed() {
        let (good, _) = protection();
        let with = |change: &dyn Fn(&mut Encryption)| {
            let mut bad = good.clone();
            change(&mut bad);
            bad
        };
        for bad in [
            with(&|e| e.scheme = "rot13".into()),
            with(&|e| e.salt = "zz".into()),
            with(&|e| e.nonce = "00".repeat(8)),
            with(&|e| e.check = "abc".into()),
            with(&|e| e.memory_kib = u32::MAX),
            with(&|e| e.memory_kib = 0),
            with(&|e| e.passes = 0),
            with(&|e| e.passes = 1000),
        ] {
            assert!(bad.validate().is_err(), "{bad:?}");
            assert!(matches!(bad.unlock("correct-horse-battery"), Err(KeyError::Damaged(_))), "{bad:?}");
        }
    }

    #[test]
    fn keys_are_checked_before_a_build_uses_them() {
        assert!(check_key("K7QF-3MWD-9XPA-H2TY-6NBC").is_ok());
        assert!(check_key("abcdefgh").is_ok());
        assert!(check_key("short").unwrap_err().contains("too short"));
        assert!(check_key(&"a".repeat(129)).unwrap_err().contains("too long"));
        for bad in ["has space", "quote\"d1234", "back\\slash1", "ümlaut-key1", "new\nline123"] {
            assert!(check_key(bad).is_err(), "{bad:?}");
        }
        assert!(Encryption::create("short").is_err());
    }

    #[test]
    fn generated_keys_are_readable_and_valid() {
        let key = generate_key().unwrap();
        assert_eq!(key.len(), 24, "{key}");
        assert_eq!(key.matches('-').count(), 4);
        assert!(check_key(&key).is_ok());
        assert!(key.chars().all(|c| c == '-' || ("ABCDEFGHJKLMNPQRSTUVWXYZ23456789".contains(c))), "{key}");
        assert_ne!(key, generate_key().unwrap());
    }

    #[test]
    fn hex_round_trips_and_rejects_junk() {
        assert_eq!(hex(&[0, 15, 255]), "000fff");
        assert_eq!(unhex::<3>("000fff"), Some([0, 15, 255]));
        assert_eq!(unhex::<3>("000ff"), None);
        assert_eq!(unhex::<3>("00zfff"), None);
        assert_eq!(unhex::<2>("é1é1"), None);
    }

    #[test]
    fn unlocked_keys_are_never_printed() {
        let (_, unlocked) = protection();
        assert_eq!(format!("{unlocked:?}"), "Unlocked(..)");
    }

    /// Setup files already in the wild were locked with an older Argon2
    /// build; this value was computed with argon2 0.5 and must never change.
    #[test]
    fn key_derivation_matches_setups_already_released() {
        let unlocked = derive("correct horse", b"0123456789abcdef", 64, 2).unwrap();
        assert_eq!(check_value(&unlocked), "8cf3737dbfe980985526c929c7c8298f3e4ab2e08b4df5d926923a71e54db2dc");
    }
}
