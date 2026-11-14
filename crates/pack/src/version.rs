//! Comparing app versions, to tell an update from a downgrade.
//!
//! Versions are compared like SemVer, but forgiving: any number of dotted
//! numbers (`1.2`, `2024.10.3.1`), an optional leading `v`, and an optional
//! pre-release after `-` that sorts before the release (`2.0.0-beta.2` <
//! `2.0.0`). Build metadata after `+` is ignored. Text that isn't a version
//! at all compares as unknown, and setup then treats it as an update.

use std::cmp::Ordering;

/// How `a` compares to `b`, or None when either isn't a version.
pub fn compare(a: &str, b: &str) -> Option<Ordering> {
    let (a_core, a_pre) = split(a)?;
    let (b_core, b_pre) = split(b)?;
    let len = a_core.len().max(b_core.len());
    for index in 0..len {
        let ordering = a_core.get(index).unwrap_or(&0).cmp(b_core.get(index).unwrap_or(&0));
        if ordering != Ordering::Equal {
            return Some(ordering);
        }
    }
    Some(match (a_pre, b_pre) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(a), Some(b)) => compare_pre(a, b),
    })
}

/// "v1.2.3-beta.1+build" → ([1, 2, 3], Some("beta.1")).
fn split(version: &str) -> Option<(Vec<u64>, Option<&str>)> {
    let version = version.trim();
    let version = version.strip_prefix(['v', 'V']).unwrap_or(version);
    let version = version.split('+').next().unwrap_or(version);
    let (core, pre) = match version.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (version, None),
    };
    let numbers: Option<Vec<u64>> = core.split('.').map(|part| part.parse().ok()).collect();
    numbers.filter(|numbers| !numbers.is_empty()).map(|numbers| (numbers, pre))
}

/// Pre-release parts: numbers compare as numbers and sort before words.
fn compare_pre(a: &str, b: &str) -> Ordering {
    let mut a_parts = a.split('.');
    let mut b_parts = b.split('.');
    loop {
        match (a_parts.next(), b_parts.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(a), Some(b)) => {
                let ordering = match (a.parse::<u64>(), b.parse::<u64>()) {
                    (Ok(a), Ok(b)) => a.cmp(&b),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => a.cmp(b),
                };
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Ordering::*;

    #[test]
    fn versions_compare_like_people_expect() {
        assert_eq!(compare("1.2.10", "1.2.9"), Some(Greater));
        assert_eq!(compare("1.2", "1.2.0"), Some(Equal));
        assert_eq!(compare("v2.0.0", "1.9.9"), Some(Greater));
        assert_eq!(compare("2.0.0-beta.2", "2.0.0"), Some(Less));
        assert_eq!(compare("2.0.0-beta.10", "2.0.0-beta.2"), Some(Greater));
        assert_eq!(compare("2.0.0-alpha", "2.0.0-beta"), Some(Less));
        assert_eq!(compare("1.0.0+build.5", "1.0.0"), Some(Equal));
        assert_eq!(compare("2024.10.3.1", "2024.10.3"), Some(Greater));
        assert_eq!(compare("nightly", "1.0"), None);
    }
}
