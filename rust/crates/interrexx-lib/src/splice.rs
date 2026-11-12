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

use std::collections::{BTreeMap, BTreeSet};
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
    /// Which side of the class the member is on: `true` for the class side.
    pub class_side: bool,
    /// Why nothing needs it.
    pub reason: &'a str,
    /// How many places across all spliced hosts send a message of the
    /// member's name (a message term, or a word of a string literal), as
    /// counted when the entry was reviewed. Those sends go to something else
    /// (a common name such as `init`). The build fails when the count differs,
    /// so a new send of the name is seen, and a count that fell is tightened.
    pub sent_sites: usize,
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
    /// A replacement file declares no class, so it replaces nothing.
    NoClass { file: String },
    /// Two replacement blocks declare one class.
    DuplicateReplacement { class: String },
    /// A host does not parse, as spliced or as given. `file` and `line` are
    /// the replacement file's when the position is inside a splice. Otherwise
    /// they are the host's name and a line of the host as spliced, which is
    /// below where the host's own line was whenever a replacement block that
    /// lies above it is longer than what it replaced.
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
    /// A `Dropped` entry whose count of send sites is not what the spliced
    /// hosts hold.
    DroppedSendCount {
        class: String,
        member: String,
        expected: usize,
        found: usize,
    },
    /// A `Dropped` entry for a member the replacement keeps.
    DroppedKept {
        class: String,
        member: String,
        class_side: bool,
    },
    /// A `Dropped` entry that matches no member a replacement drops.
    UnusedDropped {
        class: String,
        member: String,
        class_side: bool,
    },
    /// A `Dropped` entry with no reason.
    EmptyReason {
        class: String,
        member: String,
        class_side: bool,
    },
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
            SpliceError::NoClass { file } => write!(f, "{file}: declares no class"),
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
            SpliceError::DroppedSendCount {
                class,
                member,
                expected,
                found,
            } => write!(
                f,
                "{class}~{member}: allowed to be dropped with {expected} send sites of its \
                 name, and the hosts hold {found}; review them and set the entry to {found}"
            ),
            SpliceError::DroppedKept {
                class,
                member,
                class_side,
            } => write!(
                f,
                "{class}~{member} ({}): an entry allows dropping it, and the replacement \
                 keeps it",
                side_name(*class_side)
            ),
            SpliceError::UnusedDropped {
                class,
                member,
                class_side,
            } => write!(
                f,
                "{class}~{member} ({}): an entry allows dropping it, and no replaced \
                 class drops it",
                side_name(*class_side)
            ),
            SpliceError::EmptyReason {
                class,
                member,
                class_side,
            } => write!(
                f,
                "{class}~{member} ({}): the entry has no reason",
                side_name(*class_side)
            ),
        }
    }
}

impl std::error::Error for SpliceError {}

fn side_name(class_side: bool) -> &'static str {
    if class_side { "class" } else { "instance" }
}

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
        DirectiveKind::Method(method) => {
            let getter = method.name.to_ascii_uppercase();
            if method.attribute {
                // `::METHOD x ATTRIBUTE` defines the getter and the setter.
                let mut setter = getter.clone();
                setter.push(b'=');
                Some(vec![
                    (method.class_method, getter),
                    (method.class_method, setter),
                ])
            } else {
                Some(vec![(method.class_method, getter)])
            }
        }
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
        // A constant is found on the class and on its instances.
        DirectiveKind::Constant(constant) => {
            let name = constant.name.to_ascii_uppercase();
            Some(vec![(false, name.clone()), (true, name)])
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
    if blocks.is_empty() {
        return Err(SpliceError::NoClass {
            file: file.to_string(),
        });
    }
    for (index, block) in blocks.iter_mut().enumerate() {
        let from = starts[index];
        // A block ends where the next class directive starts, so a clause
        // that shares a line with that directive stays with its own block.
        let to = starts.get(index + 1).copied().unwrap_or(text.len());
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
#[cfg(test)]
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

/// How many times a host sends each upcased name as a message: a name after
/// `~` or `~~`, and each word of a string literal outside the directives that
/// declare things (message names are passed as strings to `SEND`, `START`,
/// `METHOD`, `INTERPRET` and the like). A word is a run of the characters a
/// symbol or a setter's `=` is made of, so `'o~hello'` holds `O` and `HELLO`.
/// A send by an operator (`"="`, `"[]"`) is found only as a literal's word.
fn sent_names(bytes: &[u8]) -> BTreeMap<Vec<u8>, usize> {
    let source = ProgramSource::new(bytes.to_vec(), SourceKind::Program);
    let mut names: BTreeMap<Vec<u8>, usize> = BTreeMap::new();
    let Ok(scanned) = scan(&source) else {
        return names;
    };
    let is_word = |b: &u8| b.is_ascii_alphanumeric() || b"._!?@#$=".contains(b);
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
                    *names.entry(name.as_bytes().to_vec()).or_default() += 1;
                }
                if after_colons {
                    skip_literals = !matches!(name, "CONSTANT");
                }
            }
            TokenKind::Literal { value } if !skip_literals => {
                for word in value.split(|b| !is_word(b)).filter(|w| !w.is_empty()) {
                    *names.entry(word.to_ascii_uppercase()).or_default() += 1;
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

/// How many send sites `sent` holds for `member`'s name, and for the
/// attribute's it is the setter of.
fn send_sites(sent: &BTreeMap<Vec<u8>, usize>, member: &[u8]) -> usize {
    let count = |name: &[u8]| sent.get(name).copied().unwrap_or(0);
    count(member) + member.strip_suffix(b"=").map_or(0, count)
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
    for entry in dropped {
        if entry.reason.is_empty() {
            return Err(SpliceError::EmptyReason {
                class: entry.class.to_string(),
                member: entry.member.to_string(),
                class_side: entry.class_side,
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
        // A host that declares one class twice does not parse, so `first` is
        // the only declaration.
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
    let mut used = vec![false; dropped.len()];
    let mut dropped_here: Vec<(&Block, MemberKey, &Dropped<'_>)> = Vec::new();
    for (block, &(h, c)) in blocks.iter().zip(&landing) {
        let class = &analysed[h].classes[c];
        for key in &class.members {
            let found = dropped.iter().position(|d| {
                d.class_side == key.0
                    && d.class.eq_ignore_ascii_case(&block.class)
                    && d.member.as_bytes().eq_ignore_ascii_case(&key.1)
            });
            let kept = block.members.contains(key);
            match (kept, found) {
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
                        class_side: key.0,
                    });
                }
                (false, Some(i)) => {
                    used[i] = true;
                    dropped_here.push((block, key.clone(), &dropped[i]));
                }
                (true, None) => {}
            }
        }
    }
    // An entry that settled nothing has outlived its reason.
    if let Some(i) = used.iter().position(|&u| !u) {
        return Err(SpliceError::UnusedDropped {
            class: dropped[i].class.to_string(),
            member: dropped[i].member.to_string(),
            class_side: dropped[i].class_side,
        });
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
        let mut sent: BTreeMap<Vec<u8>, usize> = BTreeMap::new();
        for spliced in &out {
            for (name, count) in sent_names(&spliced.bytes) {
                *sent.entry(name).or_default() += count;
            }
        }
        for (block, key, entry) in dropped_here {
            let found = send_sites(&sent, &key.1);
            if found != entry.sent_sites {
                return Err(SpliceError::DroppedSendCount {
                    class: block.class.clone(),
                    member: text_of(&key.1),
                    expected: entry.sent_sites,
                    found,
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
