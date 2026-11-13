//! Reading the app's `.ico` for a logo: modern icons keep their large
//! images as PNG, which the setup window can draw as is.

pub const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// The largest PNG image inside an `.ico`, if it has one.
pub fn largest_png(ico: &[u8]) -> Option<Vec<u8>> {
    let word = |at: usize| ico.get(at..at + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let dword = |at: usize| ico.get(at..at + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    if word(0)? != 0 || word(2)? != 1 {
        return None;
    }
    let count = usize::from(word(4)?);
    let mut best: Option<(u32, &[u8])> = None;
    for index in 0..count {
        let entry = 6 + index * 16;
        // 0 in the directory means 256.
        let width = match *ico.get(entry)? {
            0 => 256,
            width => u32::from(width),
        };
        let size = usize::try_from(dword(entry + 8)?).ok()?;
        let offset = usize::try_from(dword(entry + 12)?).ok()?;
        let Some(image) = offset.checked_add(size).and_then(|end| ico.get(offset..end)) else {
            continue;
        };
        if image.starts_with(PNG_SIGNATURE) && best.is_none_or(|(best_width, _)| width > best_width) {
            best = Some((width, image));
        }
    }
    best.map(|(_, image)| image.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ico(entries: &[(u8, &[u8])]) -> Vec<u8> {
        let mut out = vec![0, 0, 1, 0];
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        let mut offset = 6 + entries.len() * 16;
        for (width, data) in entries {
            out.extend_from_slice(&[*width, *width, 0, 0, 1, 0, 32, 0]);
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(offset as u32).to_le_bytes());
            offset += data.len();
        }
        for (_, data) in entries {
            out.extend_from_slice(data);
        }
        out
    }

    #[test]
    fn picks_the_largest_png() {
        let small = [PNG_SIGNATURE, b"small"].concat();
        let large = [PNG_SIGNATURE, b"large"].concat();
        let file = ico(&[(32, &small), (0, &large), (48, b"BMPDATA")]);
        assert_eq!(largest_png(&file), Some(large));
    }

    #[test]
    fn bitmaps_and_damage_give_nothing() {
        assert_eq!(largest_png(&ico(&[(32, b"BMPDATA")])), None);
        assert_eq!(largest_png(b"not an icon"), None);
        let mut cut = ico(&[(32, &[PNG_SIGNATURE, b"x"].concat())]);
        cut.truncate(cut.len() - 4);
        assert_eq!(largest_png(&cut), None);
    }
}
