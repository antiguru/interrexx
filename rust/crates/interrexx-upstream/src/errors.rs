//! The error message catalogue's interface: which messages exist, their
//! numbers and symbolic names, and how many inserts each takes and shows.
//! The texts are not part of the table.

use std::fmt::Write as _;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::header;

/// Messages the oracle raises with more inserts than their text uses, each with
/// that count. The extra inserts reach a trapped condition's `ADDITIONAL`.
pub const ARITY_OVERRIDES: &[((u16, u16), usize)] = &[
    // CALL with a namespace: the token after `ns:` is named.
    ((20, 922), 1),
    // A clause that does not end where it should: the token found is named.
    ((27, 1), 1),
    // An expression term that cannot be completed: the token the term
    // started at is named.
    ((89, 1), 1),
    // The directive errors below name the token found (for a class
    // reference, the token at which it was parsed).
    ((19, 902), 1),
    ((19, 903), 1),
    ((19, 904), 1),
    ((19, 905), 1),
    ((19, 906), 1),
    ((19, 907), 1),
    ((19, 908), 1),
    ((19, 913), 1),
    ((19, 914), 1),
    ((19, 915), 1),
    ((19, 916), 1),
    ((19, 917), 1),
    ((19, 918), 1),
    ((19, 919), 1),
    ((19, 920), 1),
    ((19, 921), 1),
    ((20, 924), 1),
    ((20, 925), 1),
    ((20, 926), 1),
    ((20, 935), 1),
    // A `::CONSTANT` outside a class: the constant's name, in upper case, as
    // the oracle answers.
    ((99, 906), 1),
];

/// One catalogue message as the source gives it, text included.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub major: u16,
    pub sub: u16,
    pub number: u16,
    pub symbol: String,
    /// The rendered text, with `&k` marking insert `k`.
    pub text: String,
}

/// The highest `k` of any `&k` in `text`, or zero for none.
pub fn highest_insert(text: &str) -> usize {
    let bytes = text.as_bytes();
    bytes
        .windows(2)
        .filter(|pair| pair[0] == b'&' && pair[1].is_ascii_digit())
        .map(|pair| usize::from(pair[1] - b'0'))
        .max()
        .unwrap_or(0)
}

/// The distinct `k` of every `&k` in `text`, ascending.
fn rendered_positions(text: &str) -> Vec<usize> {
    let mut seen: Vec<usize> = text
        .as_bytes()
        .windows(2)
        .filter(|pair| pair[0] == b'&' && pair[1].is_ascii_digit())
        .map(|pair| usize::from(pair[1] - b'0'))
        .collect();
    seen.sort_unstable();
    seen.dedup();
    seen
}

/// Which `<Message>`/`<SubMessage>` child we are currently accumulating.
#[derive(PartialEq)]
enum Field {
    None,
    Code,
    Subcode,
    Number,
    Symbol,
    Text,
}

fn apply_text(stack: &mut [Message], field: &Field, raw: &str) {
    if let Some(m) = stack.last_mut() {
        match field {
            Field::Code => m.major = parse_num(raw, "Code"),
            Field::Subcode => m.sub = parse_num(raw, "Subcode"),
            Field::Number => m.number = parse_num(raw, "MessageNumber"),
            Field::Symbol => m.symbol.push_str(raw.trim()),
            Field::Text => m.text.push_str(raw),
            Field::None => {}
        }
    }
}

fn parse_num(raw: &str, what: &str) -> u16 {
    raw.trim()
        .parse()
        .unwrap_or_else(|e| panic!("{what} {raw:?} is not a number: {e}"))
}

/// Parses the catalogue's XML into its messages, in the order each one ends.
pub fn parse_messages(xml: &str) -> Vec<Message> {
    let mut reader = Reader::from_str(xml);
    let mut messages: Vec<Message> = Vec::new();

    // A <Message> may contain <Subcodes><SubMessage>..., and a SubMessage has
    // the same child element names as its parent. Keep a stack so a
    // submessage's <Code> never overwrites the major's.
    let mut stack: Vec<Message> = Vec::new();
    let mut field = Field::None;
    // <Text> holds mixed content: character data interleaved with <q>, <sq/>,
    // <dq/> and <Sub/>. Accumulate rendered pieces rather than raw text.
    let mut depth_in_text = 0usize;

    loop {
        match reader.read_event() {
            Err(e) => panic!("malformed message catalogue: {e}"),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                "Message" | "SubMessage" => {
                    stack.push(Message::default());
                    field = Field::None;
                }
                "Code" => field = Field::Code,
                "Subcode" => field = Field::Subcode,
                "MessageNumber" => field = Field::Number,
                "SymbolicName" => field = Field::Symbol,
                "Text" => {
                    field = Field::Text;
                    depth_in_text = 0;
                }
                // Rendering rules follow the oracle's message-rendering stylesheet.
                // <q> keeps its quotes -- it is not documentation-only markup.
                "q" if field == Field::Text => {
                    depth_in_text += 1;
                    if let Some(m) = stack.last_mut() {
                        m.text.push('"');
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(e)) if field == Field::Text => {
                let rendered = match e.name().as_ref() {
                    "Sub" => {
                        let pos = e
                            .attributes()
                            .flatten()
                            .find(|a| a.key.as_ref() == "position")
                            .map(|a| a.value.into_owned())
                            .expect("every <Sub> carries a position");
                        format!("&{pos}")
                    }
                    "sq" => "'".to_string(),
                    "dq" => "\"".to_string(),
                    _ => String::new(),
                };
                if let Some(m) = stack.last_mut() {
                    m.text.push_str(&rendered);
                }
            }
            Ok(Event::Text(t)) => {
                // Entity references arrive as separate `GeneralRef` events,
                // so `t` is already literal character data.
                apply_text(&mut stack, &field, &t);
            }
            Ok(Event::GeneralRef(r)) => {
                let resolved = match r.resolve_char_ref().expect("well-formed char ref") {
                    Some(c) => c.to_string(),
                    None => match &*r {
                        "lt" => "<".to_string(),
                        "gt" => ">".to_string(),
                        "amp" => "&".to_string(),
                        "apos" => "'".to_string(),
                        "quot" => "\"".to_string(),
                        other => panic!("unknown entity &{other}; in the message catalogue"),
                    },
                };
                apply_text(&mut stack, &field, &resolved);
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                "q" if field == Field::Text && depth_in_text > 0 => {
                    depth_in_text -= 1;
                    if let Some(m) = stack.last_mut() {
                        m.text.push('"');
                    }
                }
                "Message" | "SubMessage" => {
                    let done = stack.pop().expect("balanced start/end");
                    if done.symbol.is_empty() {
                        panic!("message {}.{:03} has no SymbolicName", done.major, done.sub);
                    }
                    if done.text.is_empty() {
                        panic!("message {}.{:03} has no Text", done.major, done.sub);
                    }
                    messages.push(done);
                    field = Field::None;
                }
                "Code" | "Subcode" | "MessageNumber" | "SymbolicName" | "Text" => {
                    field = Field::None;
                }
                _ => {}
            },
            _ => {}
        }
    }

    if messages.is_empty() {
        panic!("no messages parsed from the catalogue");
    }
    // A silently colliding key would let a harness report false
    // conformance, so refuse to emit a table that has one.
    let mut keys: Vec<(u16, u16)> = messages.iter().map(|m| (m.major, m.sub)).collect();
    keys.sort_unstable();
    for pair in keys.windows(2) {
        if pair[0] == pair[1] {
            panic!("duplicate message key {}.{:03}", pair[0].0, pair[0].1);
        }
    }
    messages
}

/// One row of the checked-in table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub major: u16,
    pub sub: u16,
    pub number: u16,
    pub symbol: String,
    /// How many inserts the message takes: the override, or the text's
    /// highest `&k`.
    pub inserts: usize,
    /// The `k` of every insert the text shows, ascending. Smaller than
    /// `1..=inserts` for an override row or a text that skips a position.
    pub rendered: Vec<usize>,
}

impl Row {
    fn from_message(m: &Message) -> Row {
        let inserts = ARITY_OVERRIDES
            .iter()
            .find(|(code, _)| *code == (m.major, m.sub))
            .map_or_else(|| highest_insert(&m.text), |&(_, n)| n);
        Row {
            major: m.major,
            sub: m.sub,
            number: m.number,
            symbol: m.symbol.clone(),
            inserts,
            rendered: rendered_positions(&m.text),
        }
    }
}

/// The whole errors table, in the catalogue's order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    pub rows: Vec<Row>,
}

impl Table {
    /// Derives the table from the catalogue's XML.
    pub fn parse(xml: &str) -> Table {
        Table::from_messages(&parse_messages(xml))
    }

    /// Derives the table from parsed messages.
    pub fn from_messages(messages: &[Message]) -> Table {
        Table {
            rows: messages.iter().map(Row::from_message).collect(),
        }
    }

    /// The table as checked-in text, headed with the source revision.
    pub fn to_tsv(&self, sha: &str) -> String {
        let mut out = header(sha);
        out.push_str(
            "# major<TAB>sub<TAB>number<TAB>symbol<TAB>inserts<TAB>rendered\n\
             # inserts is how many inserts the message takes. rendered lists the insert\n\
             # positions its text shows, comma separated, or `-` for none.\n",
        );
        for row in &self.rows {
            let rendered = if row.rendered.is_empty() {
                "-".to_string()
            } else {
                row.rendered
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            };
            writeln!(
                out,
                "{}\t{}\t{}\t{}\t{}\t{rendered}",
                row.major, row.sub, row.number, row.symbol, row.inserts
            )
            .expect("writing to a String cannot fail");
        }
        out
    }

    /// Reads a table back from its checked-in text.
    pub fn from_tsv(text: &str) -> Table {
        let mut rows = Vec::new();
        for (index, line) in text.lines().enumerate() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let at = || format!("errors table line {}: {line:?}", index + 1);
            let fields: Vec<&str> = line.split('\t').collect();
            let [major, sub, number, symbol, inserts, rendered] = fields.as_slice() else {
                panic!("malformed row, {}", at());
            };
            let num = |field: &str| -> usize {
                field
                    .parse()
                    .unwrap_or_else(|_| panic!("{field:?} is not a number, {}", at()))
            };
            let small = |field: &str| -> u16 {
                u16::try_from(num(field)).unwrap_or_else(|_| panic!("{field:?} too big, {}", at()))
            };
            let rendered = if *rendered == "-" {
                Vec::new()
            } else {
                rendered.split(',').map(num).collect()
            };
            rows.push(Row {
                major: small(major),
                sub: small(sub),
                number: small(number),
                symbol: (*symbol).to_string(),
                inserts: num(inserts),
                rendered,
            });
        }
        Table { rows }
    }
}

#[cfg(test)]
mod tests;
