//! Line arithmetic over program bytes, shared by the splice and the library's
//! error sites.

/// The 1-based line of byte `offset` in `bytes`. A line ends at LF, at CR, or
/// at a CR directly followed by LF.
pub fn line_at(bytes: &[u8], offset: usize) -> usize {
    1 + line_breaks(&bytes[..offset.min(bytes.len())])
}

/// How many line ends `bytes` holds.
pub fn line_breaks(bytes: &[u8]) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => count += 1,
            b'\r' => {
                count += 1;
                if bytes.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    count
}

/// The byte offset where 1-based line `line` of `bytes` starts, or `None` past
/// the last line.
pub fn line_start(bytes: &[u8], line: usize) -> Option<usize> {
    let mut at = 0;
    let mut current = 1;
    while current < line {
        let rest = &bytes[at..];
        let step = rest.iter().position(|&b| b == b'\n' || b == b'\r')?;
        at += step;
        if bytes[at] == b'\r' && bytes.get(at + 1) == Some(&b'\n') {
            at += 1;
        }
        at += 1;
        current += 1;
    }
    (at <= bytes.len()).then_some(at)
}
