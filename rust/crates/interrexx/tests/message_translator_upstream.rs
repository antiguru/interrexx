//! The translator (`support/messages.rs`) on the oracle's real message
//! templates, read from `$REXX_UPSTREAM` at run time. Needs the pinned
//! upstream tree; `message_translator.rs` holds the tests that need nothing.
//!
//! While this crate's texts equal the oracle's, the translator is the
//! identity on every message line, which keeps the oracle comparisons
//! green. Whatever our texts are, every template of the oracle must be
//! recognised and mapped; the tests with a stand-in for our texts fix that.

mod support;

use support::messages::{self, Entry, Translator};

/// The oracle's messages, parsed from the pinned tree.
fn upstream() -> Vec<interrexx_upstream::errors::Message> {
    let root = std::env::var_os("REXX_UPSTREAM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| env!("REXX_UPSTREAM").into());
    let path = root.join("interpreter/messages/rexxmsg.xml");
    let xml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    interrexx_upstream::errors::parse_messages(&xml)
}

/// `text` with insert `k` replaced by `i<k>`.
fn filled(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('&', Some(d)) if ('1'..='9').contains(d) => {
                out.push('i');
                out.push(*d);
                chars.next();
            }
            _ => out.push(c),
        }
    }
    out
}

/// The lines the oracle prints for a message, as `(line, frame)`: the
/// line, and the part of it before the text.
fn lines_of(m: &interrexx_upstream::errors::Message) -> Vec<(String, String)> {
    let text = filled(&m.text);
    let mut lines = Vec::new();
    if (m.major, m.sub) == (101, 4) {
        return lines;
    }
    if m.sub != 0 {
        let frame = format!("Error {}.{}:  ", m.major, m.sub);
        lines.push((format!("{frame}{text}"), frame));
    } else {
        let frame = format!("Error {} running /w/p.rex line 7:  ", m.major);
        lines.push((format!("{frame}{text}"), frame));
        let bare = format!("Error {}:  ", m.major);
        lines.push((format!("{bare}{text}"), bare));
    }
    lines
}

/// The standard translator maps each of the oracle's lines to the same line
/// with this crate's text. While the two texts are equal that is the
/// identity, which is what keeps the oracle comparisons unchanged.
#[test]
fn the_standard_translator_maps_the_oracles_lines_to_ours() {
    let theirs = upstream();
    assert!(theirs.len() > 600, "{} messages read", theirs.len());
    for m in &theirs {
        let ours = interrexx_inventory::errors::lookup(m.major, m.sub)
            .unwrap_or_else(|| panic!("no text of ours for {}.{}", m.major, m.sub));
        for (line, frame) in lines_of(m) {
            let bytes = format!("{line}\n").into_bytes();
            let want = format!("{frame}{}\n", filled(ours.text));
            assert_eq!(
                String::from_utf8_lossy(&messages::oracle_output(&bytes)),
                want,
                "{}.{}",
                m.major,
                m.sub
            );
        }
    }
}

#[test]
fn every_template_of_the_oracle_is_recognised_and_mapped_to_ours() {
    let theirs = upstream();
    // A stand-in for our texts: the oracle's, marked.
    let marked: Vec<String> = theirs.iter().map(|m| format!("~ {}", m.text)).collect();
    let up: Vec<Entry<'_>> = theirs
        .iter()
        .map(|m| (m.major, m.sub, m.text.as_str()))
        .collect();
    let ours: Vec<Entry<'_>> = theirs
        .iter()
        .zip(&marked)
        .map(|(m, text)| (m.major, m.sub, text.as_str()))
        .collect();
    let translator = Translator::new(&up, &ours);
    let mut mapped = 0;
    for m in &theirs {
        for (line, frame) in lines_of(m) {
            let out = translator.translate(format!("{line}\n").as_bytes());
            let want = format!("{frame}~ {}\n", filled(&m.text));
            assert_eq!(
                String::from_utf8_lossy(&out.bytes),
                want,
                "{}.{}",
                m.major,
                m.sub
            );
            assert!(out.ambiguous.is_empty(), "{}.{}", m.major, m.sub);
            mapped += 1;
        }
    }
    assert!(mapped > 600, "{mapped} lines mapped");
    // The debug error lines, on the prefix the oracle prints.
    let prefix = theirs
        .iter()
        .find(|m| (m.major, m.sub) == (101, 4))
        .expect("the debug prefix");
    let line = format!(
        "{} 42:   {}\n",
        prefix.text,
        filled(&find(&theirs, 42, 0).text)
    );
    let out = translator.translate(line.as_bytes());
    assert_eq!(
        String::from_utf8_lossy(&out.bytes),
        format!(
            "~ {} 42:   ~ {}\n",
            prefix.text,
            filled(&find(&theirs, 42, 0).text)
        )
    );
}

fn find(
    all: &[interrexx_upstream::errors::Message],
    major: u16,
    sub: u16,
) -> &interrexx_upstream::errors::Message {
    all.iter()
        .find(|m| (m.major, m.sub) == (major, sub))
        .unwrap_or_else(|| panic!("no message {major}.{sub}"))
}

#[test]
fn every_whole_line_text_is_recognised_and_mapped_to_ours() {
    let theirs = upstream();
    let marked: Vec<String> = theirs.iter().map(|m| format!("~ {}", m.text)).collect();
    let up: Vec<Entry<'_>> = theirs
        .iter()
        .map(|m| (m.major, m.sub, m.text.as_str()))
        .collect();
    let ours: Vec<Entry<'_>> = theirs
        .iter()
        .zip(&marked)
        .map(|(m, text)| (m.major, m.sub, text.as_str()))
        .collect();
    let translator = Translator::new(&up, &ours);
    let mut mapped = 0;
    for m in theirs
        .iter()
        .filter(|m| matches!((m.major, m.sub), (101, 5 | 18..=26) | (999, _)))
    {
        let line = filled(&m.text);
        let out = translator.translate(format!("{line}\n").as_bytes());
        assert_eq!(
            String::from_utf8_lossy(&out.bytes),
            format!("~ {line}\n"),
            "{}.{}",
            m.major,
            m.sub
        );
        mapped += 1;
    }
    assert!(mapped >= 10, "{mapped} whole-line texts mapped");
}

/// The oracle itself, through the harness: a failing program's report comes
/// back in this crate's wording (the same wording today).
#[test]
fn an_oracle_run_reports_in_our_wording() {
    let oracle = support::oracle::locate();
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("message-translator");
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let program = dir.join("fail.rex");
    std::fs::write(&program, "say 1/0\n").expect("the program");
    let outcome = oracle.run(&program);
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    let ours = interrexx_inventory::errors::lookup(42, 3).expect("42.3");
    assert!(
        stderr.contains(&format!("Error 42.3:  {}\n", ours.text)),
        "{stderr}"
    );
    let ours = interrexx_inventory::errors::lookup(42, 0).expect("42");
    assert!(
        stderr.contains(&format!(" line 1:  {}\n", ours.text)),
        "{stderr}"
    );
}
