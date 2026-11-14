//! Finding which GitHub repository a project lives in, so `bach init` can
//! fill in `[updates] github` and the user only has to publish releases.

use std::path::Path;

/// `owner/name` of the `origin` remote, when the project is a git checkout
/// whose origin is on GitHub. Looks in the folder and the ones above it.
pub fn repository(dir: &Path) -> Option<String> {
    let start = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    start.ancestors().take(6).find_map(|folder| {
        let config = std::fs::read_to_string(folder.join(".git").join("config")).ok()?;
        origin_url(&config).and_then(|url| parse_url(&url))
    })
}

/// The `url` of `[remote "origin"]` in a git config.
fn origin_url(config: &str) -> Option<String> {
    let mut in_origin = false;
    for line in config.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_origin = line.replace(' ', "").eq_ignore_ascii_case("[remote\"origin\"]");
        } else if in_origin {
            if let Some(url) = line.strip_prefix("url").map(|rest| rest.trim_start()).and_then(|rest| rest.strip_prefix('=')) {
                return Some(url.trim().to_string());
            }
        }
    }
    None
}

/// `https://github.com/o/r(.git)`, `git@github.com:o/r(.git)` and
/// `ssh://git@github.com/o/r(.git)` all give `o/r`.
fn parse_url(url: &str) -> Option<String> {
    let rest = ["https://github.com/", "http://github.com/", "git@github.com:", "ssh://git@github.com/", "git://github.com/"]
        .iter()
        .find_map(|prefix| url.strip_prefix(prefix))
        // https://user:token@github.com/o/r: drop what's before the host.
        .or_else(|| url.split_once("@github.com/").map(|(_, rest)| rest))?;
    let rest = rest.trim_end_matches('/').trim_end_matches(".git");
    let (owner, name) = rest.split_once('/')?;
    let valid = |part: &str| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    (valid(owner) && valid(name)).then(|| format!("{owner}/{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_remotes_read_the_same_way() {
        for url in [
            "https://github.com/usebach/bach.git",
            "https://github.com/usebach/bach",
            "git@github.com:usebach/bach.git",
            "ssh://git@github.com/usebach/bach.git",
            "https://leif:token@github.com/usebach/bach.git",
        ] {
            assert_eq!(parse_url(url).as_deref(), Some("usebach/bach"), "{url}");
        }
        assert_eq!(parse_url("https://gitlab.com/usebach/bach.git"), None);
        assert_eq!(parse_url("https://github.com/usebach"), None);
    }

    #[test]
    fn the_origin_is_the_one_that_counts() {
        let config = "[core]\n\tbare = false\n[remote \"upstream\"]\n\turl = https://github.com/other/fork.git\n\
                      [remote \"origin\"]\n\turl = git@github.com:me/app.git\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n";
        assert_eq!(origin_url(config).as_deref(), Some("git@github.com:me/app.git"));
        assert_eq!(origin_url("[core]\n"), None);
    }

    #[test]
    fn a_checkout_is_found_from_a_subfolder() {
        let dir = std::env::temp_dir().join(format!("bach-github-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::create_dir_all(dir.join("app/src")).unwrap();
        std::fs::write(dir.join(".git/config"), "[remote \"origin\"]\n\turl = https://github.com/me/app.git\n").unwrap();
        assert_eq!(repository(&dir.join("app/src")).as_deref(), Some("me/app"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
