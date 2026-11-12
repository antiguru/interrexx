//! The build-time splice: a replacement file's classes take the place of the
//! host program's classes of the same names.
//!
//! `build.rs` compiles this file through `#[path]` and the crate's unit tests
//! use it on synthetic hosts, so it is generic over bytes and names no host
//! file. Each replaced class's directive and member directives are cut from
//! the host and blanked to as many line breaks as they held, and the
//! replacement's text for the class is put where the class directive was. A
//! host's package, directive order and every other class are untouched.
//!
//! Every error names classes, members, files, line numbers and error codes
//! only; no source text is rendered.

use std::collections::BTreeSet;
use std::fmt;
use std::ops::Range;

use crate::lines::{line_at, line_breaks};
use interrexx_parse::{
    Access, AnnotationTarget, AttributeStyle, DirectiveKind, ProgramSource, SourceKind, TokenKind,
    parse_program, scan,
};

/// One replacement file: the name errors cite it by, and its text.
#[derive(Clone, Copy)]
pub struct Replacement<'a> {
    pub file: &'a str,
    pub text: &'a [u8],
}

/// One host program: the name errors cite it by, and its bytes.
#[derive(Clone, Copy)]
pub struct Host<'a> {
    pub name: &'a str,
    pub bytes: &'a [u8],
}

/// A host member a replacement may leave out.
#[derive(Clone, Copy)]
pub struct Dropped<'a> {
    pub class: &'a str,
    pub member: &'a str,
    /// Why nothing needs it.
    pub reason: &'a str,
    /// The member's name is also a message some host code sends to something
    /// else (a common name such as `init`), which the entry's reason has
    /// settled in review. Without this, a dropped member that is still sent is
    /// an error.
    pub sent_elsewhere: bool,
}

/// Where one replaced class's text sits in a spliced host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Splice {
    /// The class name, upcased as the parser has it.
    pub class: String,
    pub file: String,
    /// The replacement text's byte range in the spliced host.
    pub range: Range<usize>,
    /// The line of the class directive in the replacement file, 1-based.
    pub first_line: usize,
}

/// One host after the splice.
#[derive(Debug, PartialEq, Eq)]
pub struct Spliced {
    pub bytes: Vec<u8>,
    pub splices: Vec<Splice>,
}

/// Why a splice was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum SpliceError {
    /// A replacement file does not parse.
    ReplacementParse {
        file: String,
        line: usize,
        code: u16,
        sub: u16,
    },
    /// A replacement file holds something other than class blocks, at `line`.
    NotClassBlocks { file: String, line: usize },
    /// Two replacement blocks declare one class.
    DuplicateReplacement { class: String },
    /// A host does not parse, as spliced or as given. `file` and `line` are
    /// the replacement file's when the position is inside a splice, and the
    /// host's own otherwise.
    HostParse {
        file: String,
        line: usize,
        code: u16,
        sub: u16,
    },
    /// A replaced class is declared by no host.
    ClassNotFound { class: String },
    /// A replaced class is declared by more than one host.
    ClassInSeveralHosts { class: String },
    /// The classes of one file land in different hosts and one is not public.
    SplitNonPublic { file: String },
    /// A host annotates a replaced class or one of its members.
    Annotated {
        host: String,
        class: String,
        member: Option<String>,
    },
    /// A host member a replacement leaves out, with no `Dropped` entry.
    MemberDropped { class: String, member: String },
    /// A `Dropped` entry whose member is still sent.
    DroppedStillSent {
        class: String,
        member: String,
        host: String,
    },
    /// A `Dropped` entry for a member the replacement keeps.
    DroppedKept { class: String, member: String },
}

impl fmt::Display for SpliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpliceError::ReplacementParse {
                file,
                line,
                code,
                sub,
            } => write!(f, "{file}:{line}: {code}.{sub}"),
            SpliceError::NotClassBlocks { file, line } => {
                write!(f, "{file}:{line}: not a class block")
            }
            SpliceError::DuplicateReplacement { class } => {
                write!(f, "class {class} is replaced twice")
            }
            SpliceError::HostParse {
                file,
                line,
                code,
                sub,
            } => write!(f, "{file}:{line}: {code}.{sub}"),
            SpliceError::ClassNotFound { class } => {
                write!(f, "class {class} is declared by no host")
            }
            SpliceError::ClassInSeveralHosts { class } => {
                write!(f, "class {class} is declared by several hosts")
            }
            SpliceError::SplitNonPublic { file } => write!(
                f,
                "{file}: its classes land in different hosts and one is not public"
            ),
            SpliceError::Annotated {
                host,
                class,
                member: None,
            } => write!(f, "{host}: an annotation targets replaced class {class}"),
            SpliceError::Annotated {
                host,
                class,
                member: Some(member),
            } => write!(
                f,
                "{host}: an annotation targets {member} of replaced class {class}"
            ),
            SpliceError::MemberDropped { class, member } => write!(
                f,
                "{class}~{member}: the host declares it, the replacement does not, and no \
                 entry allows dropping it"
            ),
            SpliceError::DroppedStillSent {
                class,
                member,
                host,
            } => write!(
                f,
                "{class}~{member}: allowed to be dropped, but a message of that name is \
                 still sent in {host}"
            ),
            SpliceError::DroppedKept { class, member } => write!(
                f,
                "{class}~{member}: an entry allows dropping it, and the replacement keeps it"
            ),
        }
    }
}

impl std::error::Error for SpliceError {}

/// A member by the name a send finds it under: the side and the upcased name,
/// an attribute's setter spelled with its `=`.
type MemberKey = (bool, Vec<u8>);

/// One class block of a replacement file.
struct Block {
    file: String,
    class: String,
    upper: Vec<u8>,
    public: bool,
    /// The block's text, ending in a line break.
    text: Vec<u8>,
    first_line: usize,
    members: BTreeSet<MemberKey>,
}

/// One class a host declares.
struct HostClass {
    upper: Vec<u8>,
    public: bool,
    /// The class directive's range, from its start to the next directive's.
    range: Range<usize>,
    /// The ranges of its member directives, which a replacement takes over.
    member_ranges: Vec<Range<usize>>,
    members: BTreeSet<MemberKey>,
}

/// What an annotation directive targets, and the class it sits under.
struct HostAnnotation {
    class: Option<usize>,
    target: AnnotationTarget,
}

/// A host read for splicing.
struct Analysed {
    classes: Vec<HostClass>,
    annotations: Vec<HostAnnotation>,
}

/// The keys of the members a directive defines, if it defines any.
fn member_keys(kind: &DirectiveKind) -> Option<Vec<MemberKey>> {
    match kind {
        DirectiveKind::Method(method) => Some(vec![(
            method.class_method,
            method.name.to_ascii_uppercase(),
        )]),
        DirectiveKind::Attribute(attribute) => {
            let getter = attribute.name.to_ascii_uppercase();
            let mut setter = getter.clone();
            setter.push(b'=');
            let side = attribute.class_method;
            Some(match attribute.style {
                AttributeStyle::Get => vec![(side, getter)],
                AttributeStyle::Set => vec![(side, setter)],
                AttributeStyle::Both => vec![(side, getter), (side, setter)],
            })
        }
        DirectiveKind::Constant(constant) => {
            Some(vec![(false, constant.name.to_ascii_uppercase())])
        }
        _ => None,
    }
}

fn parse_failure(file: &str, bytes: &[u8], error: &interrexx_parse::ParseError) -> SpliceError {
    SpliceError::HostParse {
        file: file.to_string(),
        line: line_at(bytes, error.byte),
        code: error.code(),
        sub: error.sub(),
    }
}

/// A replacement file's class blocks.
fn blocks_of(replacement: &Replacement<'_>) -> Result<Vec<Block>, SpliceError> {
    let text = replacement.text;
    let file = replacement.file;
    let program = parse_program(text.to_vec()).map_err(|error| SpliceError::ReplacementParse {
        file: file.to_string(),
        line: line_at(text, error.byte),
        code: error.code(),
        sub: error.sub(),
    })?;
    let not_blocks = |at: usize| SpliceError::NotClassBlocks {
        file: file.to_string(),
        line: line_at(text, at),
    };
    // Main code has no place: the clauses before the first directive.
    if !program.main.instructions.is_empty() {
        let at = program.main.instructions[0].clause_span.start;
        return Err(not_blocks(at));
    }
    let directives = &program.directives;
    let mut blocks: Vec<Block> = Vec::new();
    // The line start of the next class directive ends the block before it.
    let line_start = |at: usize| {
        text[..at]
            .iter()
            .rposition(|&b| b == b'\n' || b == b'\r')
            .map_or(0, |i| i + 1)
    };
    let mut starts: Vec<usize> = Vec::new();
    for directive in directives {
        let start = directive.clause_span.start;
        match &directive.kind {
            DirectiveKind::Class(class) => {
                starts.push(start);
                blocks.push(Block {
                    file: file.to_string(),
                    class: String::from_utf8_lossy(&class.name).into_owned(),
                    upper: class.name.to_ascii_uppercase(),
                    public: class.access == Access::Public,
                    text: Vec::new(),
                    first_line: line_at(text, start),
                    members: BTreeSet::new(),
                });
            }
            kind => match (member_keys(kind), blocks.last_mut()) {
                (Some(keys), Some(block)) => block.members.extend(keys),
                _ => return Err(not_blocks(start)),
            },
        }
    }
    for (index, block) in blocks.iter_mut().enumerate() {
        let from = starts[index];
        let to = starts
            .get(index + 1)
            .map_or(text.len(), |&next| line_start(next));
        let mut body = text[from..to].to_vec();
        if !matches!(body.last(), Some(b'\n' | b'\r')) {
            body.push(b'\n');
        }
        block.text = body;
    }
    Ok(blocks)
}

/// A host's classes, their member ranges and its annotations.
fn analyse(host: &Host<'_>) -> Result<Analysed, SpliceError> {
    let program = parse_program(host.bytes.to_vec())
        .map_err(|error| parse_failure(host.name, host.bytes, &error))?;
    let directives = &program.directives;
    let end_of = |index: usize| {
        directives
            .get(index + 1)
            .map_or(host.bytes.len(), |next| next.clause_span.start)
            .min(host.bytes.len())
    };
    let mut classes: Vec<HostClass> = Vec::new();
    let mut annotations = Vec::new();
    for (index, directive) in directives.iter().enumerate() {
        let range = directive.clause_span.start..end_of(index);
        match &directive.kind {
            DirectiveKind::Class(class) => classes.push(HostClass {
                upper: class.name.to_ascii_uppercase(),
                public: class.access == Access::Public,
                range,
                member_ranges: Vec::new(),
                members: BTreeSet::new(),
            }),
            DirectiveKind::Annotate(annotate) => annotations.push(HostAnnotation {
                class: classes.len().checked_sub(1),
                target: annotate.target.clone(),
            }),
            kind => {
                // A member belongs to the last class above it, also across an
                // intervening routine; one above every class belongs to none.
                if let (Some(keys), Some(class)) = (member_keys(kind), classes.last_mut()) {
                    class.members.extend(keys);
                    class.member_ranges.push(range);
                }
            }
        }
    }
    Ok(Analysed {
        classes,
        annotations,
    })
}

/// `host` with the blocks of `blocks` that it declares put in place.
fn apply(host: &[u8], analysed: &Analysed, blocks: &[&Block]) -> Spliced {
    // (cut range, the block that goes where the cut starts)
    let mut edits: Vec<(Range<usize>, Option<&Block>)> = Vec::new();
    for block in blocks {
        let Some(class) = analysed.classes.iter().find(|c| c.upper == block.upper) else {
            continue;
        };
        edits.push((class.range.clone(), Some(block)));
        for range in &class.member_ranges {
            edits.push((range.clone(), None));
        }
    }
    edits.sort_by_key(|(range, _)| range.start);
    let mut bytes = Vec::with_capacity(host.len());
    let mut splices = Vec::new();
    let mut at = 0;
    for (range, block) in edits {
        bytes.extend_from_slice(&host[at..range.start]);
        if let Some(block) = block {
            splices.push(Splice {
                class: block.class.clone(),
                file: block.file.clone(),
                range: bytes.len()..bytes.len() + block.text.len(),
                first_line: block.first_line,
            });
            bytes.extend_from_slice(&block.text);
        }
        // The cut keeps its lines, so the host's later lines do not move
        // because of what was cut.
        bytes.extend(std::iter::repeat_n(
            b'\n',
            line_breaks(&host[range.start..range.end]),
        ));
        at = range.end;
    }
    bytes.extend_from_slice(&host[at..]);
    Spliced { bytes, splices }
}

/// Splices one host. A replacement class the host does not declare is left
/// out; `splice_all` is what checks that every class is declared once.
pub fn splice(host: &[u8], replacements: &[Replacement<'_>]) -> Result<Spliced, SpliceError> {
    let mut blocks = Vec::new();
    for replacement in replacements {
        blocks.extend(blocks_of(replacement)?);
    }
    let refs: Vec<&Block> = blocks.iter().collect();
    let analysed = analyse(&Host {
        name: "host",
        bytes: host,
    })?;
    Ok(apply(host, &analysed, &refs))
}

/// The upcased names a host sends as messages, or names in a string literal
/// outside the directives that declare things (message names are passed as
/// strings to `SEND`, `START`, `METHOD` and the like).
fn sent_names(bytes: &[u8]) -> BTreeSet<Vec<u8>> {
    let source = ProgramSource::new(bytes.to_vec(), SourceKind::Program);
    let mut names = BTreeSet::new();
    let Ok(scanned) = scan(&source) else {
        return names;
    };
    let mut after_tilde = false;
    let mut skip_literals = false;
    let mut after_colons = false;
    for token in &scanned.tokens {
        match &token.kind {
            TokenKind::Tilde | TokenKind::DTilde => {
                after_tilde = true;
                after_colons = false;
                continue;
            }
            TokenKind::DColon => {
                after_colons = true;
                after_tilde = false;
                continue;
            }
            TokenKind::Symbol { id, .. } => {
                let name = scanned.symbols.name(*id);
                if after_tilde {
                    names.insert(name.as_bytes().to_vec());
                }
                if after_colons {
                    skip_literals = !matches!(name, "CONSTANT");
                }
            }
            TokenKind::Literal { value } if !skip_literals => {
                for word in value
                    .split(|b| b.is_ascii_whitespace() || *b == b',')
                    .filter(|w| !w.is_empty())
                {
                    names.insert(word.to_ascii_uppercase());
                }
            }
            TokenKind::Eoc => skip_literals = false,
            _ => {}
        }
        after_tilde = false;
        after_colons = false;
    }
    names
}

/// Whether `sent` holds `member`'s name, or the attribute's it is the setter
/// of.
fn is_sent(sent: &BTreeSet<Vec<u8>>, member: &[u8]) -> bool {
    sent.contains(member)
        || member
            .strip_suffix(b"=")
            .is_some_and(|stem| sent.contains(stem))
}

fn text_of(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Splices every host with every replacement and runs the guards:
///
/// * a replacement file holds only class blocks and parses;
/// * each replaced class is declared by exactly one host;
/// * a file's classes share a host, or it declares no non-public class;
/// * no host annotation targets a replaced class or one of its members;
/// * a host member the replacement leaves out has a `dropped` entry, and
///   nothing sends it (all hosts are searched, as spliced);
/// * each spliced host parses.
///
/// The answer is parallel to `hosts`.
pub fn splice_all(
    hosts: &[Host<'_>],
    replacements: &[Replacement<'_>],
    dropped: &[Dropped<'_>],
) -> Result<Vec<Spliced>, SpliceError> {
    let mut blocks: Vec<Block> = Vec::new();
    for replacement in replacements {
        blocks.extend(blocks_of(replacement)?);
    }
    for (index, block) in blocks.iter().enumerate() {
        if blocks[..index].iter().any(|b| b.upper == block.upper) {
            return Err(SpliceError::DuplicateReplacement {
                class: block.class.clone(),
            });
        }
    }
    let analysed: Vec<Analysed> = hosts.iter().map(analyse).collect::<Result<_, _>>()?;

    // The one host each block lands in.
    let mut landing: Vec<(usize, usize)> = Vec::new();
    for block in &blocks {
        let mut found = analysed.iter().enumerate().filter_map(|(h, a)| {
            a.classes
                .iter()
                .position(|c| c.upper == block.upper)
                .map(|c| (h, c))
        });
        let (Some(first), None) = (found.next(), found.next()) else {
            let class = block.class.clone();
            return Err(
                match analysed
                    .iter()
                    .any(|a| a.classes.iter().any(|c| c.upper == block.upper))
                {
                    true => SpliceError::ClassInSeveralHosts { class },
                    false => SpliceError::ClassNotFound { class },
                },
            );
        };
        // A host that declares the class twice is "several" too.
        if analysed[first.0]
            .classes
            .iter()
            .filter(|c| c.upper == block.upper)
            .count()
            > 1
        {
            return Err(SpliceError::ClassInSeveralHosts {
                class: block.class.clone(),
            });
        }
        landing.push(first);
    }

    for replacement in replacements {
        let mut in_file = blocks
            .iter()
            .zip(&landing)
            .filter(|(b, _)| b.file == replacement.file);
        let mut hosts_of_file = BTreeSet::new();
        let mut non_public = false;
        for (block, &(h, c)) in &mut in_file {
            hosts_of_file.insert(h);
            non_public |= !block.public || !analysed[h].classes[c].public;
        }
        if hosts_of_file.len() > 1 && non_public {
            return Err(SpliceError::SplitNonPublic {
                file: replacement.file.to_string(),
            });
        }
    }

    for (block, &(h, c)) in blocks.iter().zip(&landing) {
        for annotation in &analysed[h].annotations {
            let hits_class = matches!(&annotation.target,
                AnnotationTarget::Class(name) if name.to_ascii_uppercase() == block.upper);
            let hits_member = annotation.class == Some(c)
                && matches!(
                    annotation.target,
                    AnnotationTarget::Method(_)
                        | AnnotationTarget::Attribute(_)
                        | AnnotationTarget::Constant(_)
                );
            if hits_class || hits_member {
                let member = match &annotation.target {
                    AnnotationTarget::Method(n)
                    | AnnotationTarget::Attribute(n)
                    | AnnotationTarget::Constant(n)
                        if hits_member =>
                    {
                        Some(text_of(n))
                    }
                    _ => None,
                };
                return Err(SpliceError::Annotated {
                    host: hosts[h].name.to_string(),
                    class: block.class.clone(),
                    member,
                });
            }
        }
    }

    // Members the host declared and the replacement does not.
    let mut dropped_here: Vec<(&Block, MemberKey, Option<&Dropped<'_>>)> = Vec::new();
    for (block, &(h, c)) in blocks.iter().zip(&landing) {
        let class = &analysed[h].classes[c];
        for key in &class.members {
            // An entry without its reason does not count: the reason is what
            // a reviewer reads.
            let entry = dropped.iter().find(|d| {
                !d.reason.is_empty()
                    && d.class.eq_ignore_ascii_case(&block.class)
                    && d.member.as_bytes().eq_ignore_ascii_case(&key.1)
            });
            let kept = block.members.contains(key);
            match (kept, entry) {
                (false, None) => {
                    return Err(SpliceError::MemberDropped {
                        class: block.class.clone(),
                        member: text_of(&key.1),
                    });
                }
                (true, Some(_)) => {
                    return Err(SpliceError::DroppedKept {
                        class: block.class.clone(),
                        member: text_of(&key.1),
                    });
                }
                (false, Some(_)) => dropped_here.push((block, key.clone(), entry)),
                (true, None) => {}
            }
        }
    }

    let mut out = Vec::new();
    for (h, host) in hosts.iter().enumerate() {
        let mine: Vec<&Block> = blocks
            .iter()
            .zip(&landing)
            .filter(|(_, (host_index, _))| *host_index == h)
            .map(|(b, _)| b)
            .collect();
        out.push(apply(host.bytes, &analysed[h], &mine));
    }

    for (host, spliced) in hosts.iter().zip(&out) {
        if let Err(error) = parse_program(spliced.bytes.clone()) {
            return Err(located(host.name, spliced, &error));
        }
    }

    if !dropped_here.is_empty() {
        let sent: Vec<BTreeSet<Vec<u8>>> = out.iter().map(|s| sent_names(&s.bytes)).collect();
        for (block, key, entry) in dropped_here {
            let Some(entry) = entry else { continue };
            if entry.sent_elsewhere {
                continue;
            }
            if let Some(h) = sent.iter().position(|names| is_sent(names, &key.1)) {
                return Err(SpliceError::DroppedStillSent {
                    class: block.class.clone(),
                    member: text_of(&key.1),
                    host: hosts[h].name.to_string(),
                });
            }
        }
    }
    Ok(out)
}

/// A parse error of a spliced host, placed in the replacement file when the
/// failing clause is inside a splice.
fn located(host: &str, spliced: &Spliced, error: &interrexx_parse::ParseError) -> SpliceError {
    let byte = error.byte;
    for splice in &spliced.splices {
        if splice.range.contains(&byte) {
            let inside = line_breaks(&spliced.bytes[splice.range.start..byte]);
            return SpliceError::HostParse {
                file: splice.file.clone(),
                line: splice.first_line + inside,
                code: error.code(),
                sub: error.sub(),
            };
        }
    }
    parse_failure(host, &spliced.bytes, error)
}

#[cfg(test)]
#[path = "splice/tests.rs"]
mod tests;
