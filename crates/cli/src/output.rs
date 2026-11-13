//! How `bach` talks: short lines, sizes people can read, errors as one
//! sentence per cause.

use std::io::{IsTerminal, Write};

/// `1536` → "1.5 KB".
pub fn size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["bytes", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} {}", if bytes == 1 { "byte" } else { "bytes" });
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

/// A labelled line: `  Output   dist\Hello-Setup-1.0.0.exe`.
pub fn field(label: &str, value: impl std::fmt::Display) {
    println!("  {label:<12} {value}");
}

/// Prints an error and its causes, each on its own line.
pub fn error(err: &anyhow::Error) {
    let mut chain = err.chain();
    if let Some(first) = chain.next() {
        eprintln!("error: {}", sentence(&first.to_string()));
    }
    for cause in chain {
        eprintln!("  {}", sentence(&cause.to_string()));
    }
}

fn sentence(text: &str) -> String {
    let text = text.trim();
    let mut chars = text.chars();
    let mut out = match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
        None => String::new(),
    };
    if !out.ends_with(['.', '!', '?']) {
        out.push('.');
    }
    out
}

/// A progress line that rewrites itself on a terminal and stays quiet in
/// logs (CI), where only the final line matters.
pub struct Progress {
    label: &'static str,
    total: u64,
    shown: Option<u32>,
    live: bool,
}

impl Progress {
    pub fn new(label: &'static str, total: u64) -> Self {
        Self { label, total: total.max(1), shown: None, live: std::io::stderr().is_terminal() }
    }

    pub fn update(&mut self, done: u64) {
        if !self.live {
            return;
        }
        let percent = (done.min(self.total) * 100 / self.total) as u32;
        if self.shown != Some(percent) {
            self.shown = Some(percent);
            eprint!("\r  {:<12} {percent:>3}%", self.label);
            let _ = std::io::stderr().flush();
        }
    }

    pub fn finish(&mut self) {
        if self.live && self.shown.is_some() {
            eprint!("\r{:<40}\r", "");
            let _ = std::io::stderr().flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_naturally() {
        assert_eq!(size(1), "1 byte");
        assert_eq!(size(512), "512 bytes");
        assert_eq!(size(1536), "1.5 KB");
        assert_eq!(size(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn errors_read_as_sentences() {
        assert_eq!(sentence("couldn't read x"), "Couldn't read x.");
        assert_eq!(sentence("Done."), "Done.");
    }
}
