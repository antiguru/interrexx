//! The `rxregexp` pattern language and its matcher, as
//! `docs/design/extensions.md` describes
//! them: a pattern compiles to a Thompson automaton, and every question the
//! `RegularExpression` methods ask is one pass of it over the subject, which
//! keeps each answer linear in the subject and the automaton (Deviation 87:
//! `(a*)*` and a set range ending at `'7F'x` terminate).
//!
//! The language is the grammar's, with standard meanings for alternation and
//! repetition; the oracle's two departures from it, D4 and D5, are not
//! reproduced.

/// One automaton state.
#[derive(Clone, Debug)]
enum State {
    /// Consumes one byte in the set, then goes to `next`.
    Byte { set: u32, next: u32 },
    /// Goes to both, consuming nothing.
    Split(u32, u32),
    /// Goes to the target, consuming nothing; patched once it is known.
    Jump(u32),
    /// The pattern has matched.
    Accept,
}

/// A byte set, one bit per byte value.
type ByteSet = [u64; 4];

fn set_has(set: &ByteSet, byte: u8) -> bool {
    set[usize::from(byte >> 6)] & (1 << (byte & 63)) != 0
}

fn set_add(set: &mut ByteSet, byte: u8) {
    set[usize::from(byte >> 6)] |= 1 << (byte & 63);
}

/// A parsed pattern.
#[derive(Clone, Debug)]
enum Node {
    /// One byte in the set.
    Set(ByteSet),
    /// Each in turn.
    Concat(Vec<Node>),
    /// Either.
    Alternate(Box<Node>, Box<Node>),
    /// Zero or more.
    Star(Box<Node>),
    /// One or more.
    Plus(Box<Node>),
    /// Exactly this many, at least 2.
    Count(Box<Node>, u32),
}

impl Node {
    /// The number of automaton states [`Builder::build`] makes for this
    /// node, saturated.
    fn size(&self) -> u64 {
        match self {
            Node::Set(_) => 1,
            Node::Concat(nodes) => nodes
                .iter()
                .fold(0, |sum, node| sum.saturating_add(node.size())),
            Node::Alternate(left, right) => {
                left.size().saturating_add(right.size()).saturating_add(1)
            }
            Node::Star(inner) | Node::Plus(inner) => inner.size().saturating_add(1),
            Node::Count(inner, count) => inner.size().saturating_mul(u64::from(*count)),
        }
    }
}

/// The most states a pattern may compile to: about 40 MB of automaton and
/// thread sets. A count past it is compile error 5 at its `{` (Deviation 87);
/// measured, the oracle answers `a{10000000}` and crashes on `a{2000000000}`
/// and on `((a{1000}){1000}){1000}`.
const STATE_BUDGET: u64 = 2_000_000;

/// A compile error: its code (1 to 6) and the position `!POS` reports.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct CompileError {
    pub(crate) code: u8,
    pub(crate) position: usize,
}

const UNEXPECTED: u8 = 1;
const MISSING_PAREN: u8 = 2;
const ILLEGAL_SET: u8 = 3;
const UNEXPECTED_END: u8 = 4;
const ILLEGAL_NUMBER: u8 = 5;
const BAD_CLASS: u8 = 6;

/// The bytes that cannot begin an atom.
fn ends_term(byte: u8) -> bool {
    matches!(byte, b')' | b'{' | b'}' | b'|' | b'*' | b'+' | b']')
}

/// The recursive-descent parser over the pattern's bytes before any NUL.
struct Parser<'a> {
    pattern: &'a [u8],
    at: usize,
    /// The index of the last `{` parsed, which a pattern over the state
    /// budget is blamed on.
    last_count: Option<usize>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.pattern.get(self.at).copied()
    }

    fn error(&self, code: u8, position: usize) -> CompileError {
        CompileError { code, position }
    }

    /// `expression ::= term [ "|" expression ]`.
    fn expression(&mut self) -> Result<Node, CompileError> {
        let term = self.term()?;
        if self.peek() == Some(b'|') {
            self.at += 1;
            let rest = self.expression()?;
            return Ok(Node::Alternate(Box::new(term), Box::new(rest)));
        }
        Ok(term)
    }

    /// `term ::= factor { factor }`.
    fn term(&mut self) -> Result<Node, CompileError> {
        let mut factors = vec![self.factor()?];
        while let Some(byte) = self.peek() {
            if ends_term(byte) {
                break;
            }
            factors.push(self.factor()?);
        }
        Ok(if factors.len() == 1 {
            factors.pop().expect("one factor")
        } else {
            Node::Concat(factors)
        })
    }

    /// `factor ::= atom [ "*" | "+" | "{" count "}" ]`: one quantifier, a
    /// second ending the term.
    fn factor(&mut self) -> Result<Node, CompileError> {
        let atom = self.atom()?;
        match self.peek() {
            Some(b'*') => {
                self.at += 1;
                Ok(Node::Star(Box::new(atom)))
            }
            Some(b'+') => {
                self.at += 1;
                Ok(Node::Plus(Box::new(atom)))
            }
            Some(b'{') => {
                let open = self.at;
                let Some(length) = self.pattern[open + 1..].iter().position(|b| *b == b'}') else {
                    return Err(self.error(UNEXPECTED_END, open));
                };
                let count = atoi(&self.pattern[open + 1..open + 1 + length]);
                if count < 2 {
                    return Err(self.error(ILLEGAL_NUMBER, open));
                }
                let count = u32::try_from(count).expect("a positive int");
                let node = Node::Count(Box::new(atom), count);
                // Deviation 87: compiling always terminates, so a count whose
                // copies would pass the budget is an illegal number.
                if node.size() > STATE_BUDGET {
                    return Err(self.error(ILLEGAL_NUMBER, open));
                }
                self.last_count = Some(open);
                self.at = open + length + 2;
                Ok(node)
            }
            _ => Ok(atom),
        }
    }

    /// `atom ::= "?" | "\" byte | "[" set "]" | "(" expression ")" | literal`.
    fn atom(&mut self) -> Result<Node, CompileError> {
        let Some(byte) = self.peek() else {
            return Err(self.error(UNEXPECTED, self.at));
        };
        match byte {
            b'?' => {
                self.at += 1;
                Ok(Node::Set([u64::MAX; 4]))
            }
            b'\\' => {
                let Some(escaped) = self.pattern.get(self.at + 1).copied() else {
                    return Err(self.error(UNEXPECTED_END, self.at + 1));
                };
                self.at += 2;
                Ok(single(escaped))
            }
            b'[' => self.set(),
            b'(' => {
                self.at += 1;
                let inner = self.expression()?;
                if self.peek() != Some(b')') {
                    return Err(self.error(MISSING_PAREN, self.at));
                }
                self.at += 1;
                Ok(inner)
            }
            _ if ends_term(byte) => Err(self.error(UNEXPECTED, self.at)),
            _ => {
                self.at += 1;
                Ok(single(byte))
            }
        }
    }

    /// `"[" set "]"`, with the error positions the description gives: just
    /// after the `[` for bad contents, after the contents for a missing `]`.
    fn set(&mut self) -> Result<Node, CompileError> {
        let open = self.at;
        let bad = |code| CompileError {
            code,
            position: open + 1,
        };
        let mut at = open + 1;
        let negated = self.pattern.get(at) == Some(&b'^');
        if negated {
            at += 1;
        }
        let first = at;
        let mut members: ByteSet = [0; 4];
        let mut items = 0usize;
        loop {
            let Some(byte) = self.pattern.get(at).copied() else {
                if items == 0 {
                    return Err(bad(ILLEGAL_SET));
                }
                return Err(self.error(ILLEGAL_SET, at));
            };
            match byte {
                b']' => break,
                b'\\' => {
                    let Some(escaped) = self.pattern.get(at + 1).copied() else {
                        return Err(bad(ILLEGAL_SET));
                    };
                    set_add(&mut members, escaped);
                    at += 2;
                }
                b':' => {
                    let name_start = at + 1;
                    let Some(length) = self.pattern[name_start..].iter().position(|b| *b == b':')
                    else {
                        return Err(bad(BAD_CLASS));
                    };
                    let name = &self.pattern[name_start..name_start + length];
                    if length > 15 {
                        return Err(bad(BAD_CLASS));
                    }
                    let Some(class) = class_named(name) else {
                        return Err(bad(BAD_CLASS));
                    };
                    for value in 0..=255u8 {
                        if class(value) {
                            set_add(&mut members, value);
                        }
                    }
                    at = name_start + length + 1;
                }
                b'-' if at == first => {
                    set_add(&mut members, b'-');
                    at += 1;
                }
                b'-' => {
                    let Some(high) = self.pattern.get(at + 1).copied().filter(|b| *b != b']')
                    else {
                        return Err(bad(ILLEGAL_SET));
                    };
                    let low = self.pattern[at - 1];
                    let signed = |b: u8| i16::from(i8::from_ne_bytes([b]));
                    let (lo, hi) = {
                        let (a, b) = (signed(low), signed(high));
                        (a.min(b), a.max(b))
                    };
                    for value in lo + 1..=hi {
                        let byte = i8::try_from(value).expect("a signed byte").to_ne_bytes()[0];
                        set_add(&mut members, byte);
                    }
                    at += 2;
                }
                other => {
                    set_add(&mut members, other);
                    at += 1;
                }
            }
            items += 1;
        }
        if items == 0 {
            return Err(bad(ILLEGAL_SET));
        }
        if negated {
            for word in &mut members {
                *word = !*word;
            }
        }
        self.at = at + 1;
        Ok(Node::Set(members))
    }
}

/// The node matching exactly `byte`.
fn single(byte: u8) -> Node {
    let mut set = [0; 4];
    set_add(&mut set, byte);
    Node::Set(set)
}

/// C's `atoi` as glibc has it: `strtol`, saturated to a `long`, then
/// truncated to an `int`. Measured, oracle: `a{4294967298}` is `a{2}`.
fn atoi(text: &[u8]) -> i64 {
    let mut at = text
        .iter()
        .position(|byte| !matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
        .unwrap_or(text.len());
    let negative = text.get(at) == Some(&b'-');
    if matches!(text.get(at), Some(b'+' | b'-')) {
        at += 1;
    }
    let mut value: i64 = 0;
    for byte in &text[at..] {
        if !byte.is_ascii_digit() {
            break;
        }
        let digit = i64::from(byte - b'0');
        value = if negative {
            value.saturating_mul(10).saturating_sub(digit)
        } else {
            value.saturating_mul(10).saturating_add(digit)
        };
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the C `(int)` cast of a `long`, which is the conversion measured"
    )]
    let truncated = value as i32;
    i64::from(truncated)
}

/// The C locale's classification a class name names, caselessly.
fn class_named(name: &[u8]) -> Option<fn(u8) -> bool> {
    let upper = name.to_ascii_uppercase();
    Some(match &upper[..] {
        b"ALPHA" => |b: u8| b.is_ascii_alphabetic(),
        b"LOWER" => |b: u8| b.is_ascii_lowercase(),
        b"UPPER" => |b: u8| b.is_ascii_uppercase(),
        b"ALNUM" => |b: u8| b.is_ascii_alphanumeric(),
        b"DIGIT" => |b: u8| b.is_ascii_digit(),
        b"XDIGIT" => |b: u8| b.is_ascii_hexdigit(),
        b"BLANK" => |b: u8| b == b' ' || b == b'\t',
        b"SPACE" => |b: u8| b == b' ' || (0x09..=0x0d).contains(&b),
        b"CNTRL" => |b: u8| b < 0x20 || b == 0x7f,
        b"PRINT" => |b: u8| (0x20..=0x7e).contains(&b),
        b"PUNCT" => |b: u8| b.is_ascii_punctuation(),
        b"GRAPH" => |b: u8| b.is_ascii_graphic(),
        _ => return None,
    })
}

/// A compiled pattern.
#[derive(Clone, Debug)]
pub(crate) struct Regex {
    states: Vec<State>,
    sets: Vec<ByteSet>,
    start: u32,
}

/// What compiling a pattern answers: the automaton and the number of
/// pattern bytes the parse consumed, or the error.
pub(crate) fn compile(pattern: &[u8]) -> Result<(Regex, usize), CompileError> {
    let end = pattern
        .iter()
        .position(|b| *b == 0)
        .unwrap_or(pattern.len());
    let mut parser = Parser {
        pattern: &pattern[..end],
        at: 0,
        last_count: None,
    };
    let node = parser.expression()?;
    // Only a count multiplies states; without one the automaton is linear
    // in the pattern.
    if let Some(open) = parser.last_count
        && node.size() > STATE_BUDGET
    {
        return Err(CompileError {
            code: ILLEGAL_NUMBER,
            position: open,
        });
    }
    let mut builder = Builder {
        states: Vec::new(),
        sets: Vec::new(),
        shared: std::collections::HashMap::new(),
    };
    let accept = builder.push(State::Accept);
    debug_assert_eq!(accept, ACCEPT);
    let start = builder.build(&node, accept);
    Ok((
        Regex {
            states: builder.states,
            sets: builder.sets,
            start,
        },
        parser.at,
    ))
}

/// Builds the automaton back to front: each node is compiled with the state
/// it continues to already known, so no patching is needed except for loops.
struct Builder {
    states: Vec<State>,
    sets: Vec<ByteSet>,
    /// Each distinct set's index in `sets`, so the copies a count makes
    /// share one.
    shared: std::collections::HashMap<ByteSet, u32>,
}

impl Builder {
    fn push(&mut self, state: State) -> u32 {
        let index = u32::try_from(self.states.len()).expect("an automaton below 2^32 states");
        self.states.push(state);
        index
    }

    /// The entry state of `node` followed by `next`.
    fn build(&mut self, node: &Node, next: u32) -> u32 {
        match node {
            Node::Set(set) => {
                let index = match self.shared.get(set) {
                    Some(index) => *index,
                    None => {
                        let index = u32::try_from(self.sets.len()).expect("below 2^32 sets");
                        self.sets.push(*set);
                        self.shared.insert(*set, index);
                        index
                    }
                };
                self.push(State::Byte { set: index, next })
            }
            Node::Concat(nodes) => nodes
                .iter()
                .rev()
                .fold(next, |after, node| self.build(node, after)),
            Node::Alternate(left, right) => {
                let left = self.build(left, next);
                let right = self.build(right, next);
                self.push(State::Split(left, right))
            }
            Node::Star(inner) => {
                let split = self.push(State::Jump(next));
                let body = self.build(inner, split);
                self.states[split as usize] = State::Split(body, next);
                split
            }
            Node::Plus(inner) => {
                let split = self.push(State::Jump(next));
                let body = self.build(inner, split);
                self.states[split as usize] = State::Split(body, next);
                body
            }
            Node::Count(inner, count) => {
                (0..*count).fold(next, |after, _| self.build(inner, after))
            }
        }
    }
}

/// A set of states, each with the earliest start that reaches it.
struct Threads {
    start_of: Vec<u32>,
    members: Vec<u32>,
}

const ABSENT: u32 = u32::MAX;

/// The accepting state's index: [`compile`] builds it first.
const ACCEPT: u32 = 0;

impl Threads {
    fn new(size: usize) -> Threads {
        Threads {
            start_of: vec![ABSENT; size],
            members: Vec::new(),
        }
    }

    fn clear(&mut self) {
        for state in self.members.drain(..) {
            self.start_of[state as usize] = ABSENT;
        }
    }

    fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

impl Regex {
    /// Adds `state` and everything it reaches without consuming, each with
    /// `start` where that is earlier than what it holds.
    fn add(&self, threads: &mut Threads, state: u32, start: u32, stack: &mut Vec<u32>) {
        stack.push(state);
        while let Some(state) = stack.pop() {
            let held = threads.start_of[state as usize];
            if held <= start {
                continue;
            }
            if held == ABSENT {
                threads.members.push(state);
            }
            threads.start_of[state as usize] = start;
            match self.states[state as usize] {
                State::Split(a, b) => {
                    stack.push(b);
                    stack.push(a);
                }
                State::Jump(target) => stack.push(target),
                State::Byte { .. } | State::Accept => {}
            }
        }
    }

    /// The states `current` moves to on `byte`.
    fn step(&self, current: &Threads, byte: u8, next: &mut Threads, stack: &mut Vec<u32>) {
        next.clear();
        for &state in &current.members {
            if let State::Byte { set, next: target } = self.states[state as usize]
                && set_has(&self.sets[set as usize], byte)
            {
                self.add(next, target, current.start_of[state as usize], stack);
            }
        }
    }

    /// The earliest start among the threads at the accepting state, which is
    /// the first state built.
    fn accepted(&self, threads: &Threads) -> Option<u32> {
        Some(threads.start_of[ACCEPT as usize]).filter(|start| *start != ABSENT)
    }

    /// Runs the automaton anchored at the start of `subject`, calling
    /// `accept(j)` for each prefix length `j` in the language, in increasing
    /// order, until it answers `true`. Answers whether it did, and the
    /// longest prefix any thread consumed.
    fn run(&self, subject: &[u8], mut accept: impl FnMut(usize) -> bool) -> (bool, usize) {
        let mut stack = Vec::new();
        let mut current = Threads::new(self.states.len());
        let mut next = Threads::new(self.states.len());
        self.add(&mut current, self.start, 0, &mut stack);
        let mut consumed = 0;
        for j in 0..=subject.len() {
            if self.accepted(&current).is_some() && accept(j) {
                return (true, j);
            }
            let Some(&byte) = subject.get(j) else {
                break;
            };
            self.step(&current, byte, &mut next, &mut stack);
            std::mem::swap(&mut current, &mut next);
            if current.is_empty() {
                break;
            }
            consumed = j + 1;
        }
        (false, consumed)
    }

    /// MAXIMAL `match`: the smallest `k` whose prefix is in the language and
    /// ends the subject or is followed by a NUL. On success the position is
    /// the subject's length or `k + 1`; on failure, the longest prefix a
    /// partial match consumed.
    pub(crate) fn match_maximal(&self, subject: &[u8]) -> (bool, usize) {
        let n = subject.len();
        let (matched, at) = self.run(subject, |k| k == n || subject[k] == 0);
        match (matched, at) {
            (true, k) if k == n => (true, n),
            (true, k) => (true, k + 1),
            (false, consumed) => (false, consumed),
        }
    }

    /// MINIMAL `match`: the shortest prefix in the language, the empty one
    /// included.
    pub(crate) fn match_minimal(&self, subject: &[u8]) -> (bool, usize) {
        self.run(subject, |_| true)
    }

    /// The leftmost start of a match, with the shortest match there: a
    /// single pass carrying each state's earliest start.
    fn leftmost_shortest(&self, haystack: &[u8]) -> Option<(usize, usize)> {
        let mut stack = Vec::new();
        let mut current = Threads::new(self.states.len());
        let mut next = Threads::new(self.states.len());
        let mut best: Option<(u32, usize)> = None;
        for i in 0..=haystack.len() {
            let here = u32::try_from(i).expect("a subject below 2^32 bytes");
            if best.is_none() {
                self.add(&mut current, self.start, here, &mut stack);
            }
            if let Some(start) = self.accepted(&current)
                && best.is_none_or(|(held, _)| start < held)
            {
                best = Some((start, i));
            }
            if let Some((held, _)) = best {
                // Only an earlier start can still change the answer.
                let Threads { start_of, members } = &mut current;
                members.retain(|state| {
                    let keep = start_of[*state as usize] < held;
                    if !keep {
                        start_of[*state as usize] = ABSENT;
                    }
                    keep
                });
                if current.is_empty() {
                    break;
                }
            }
            let Some(&byte) = haystack.get(i) else {
                break;
            };
            self.step(&current, byte, &mut next, &mut stack);
            std::mem::swap(&mut current, &mut next);
        }
        best.map(|(start, end)| (start as usize, end - start as usize))
    }

    /// `pos(haystack)`: the 1-based start of the leftmost shortest match, or
    /// 0, and the position `!POS` takes. MAXIMAL mode extends the match at
    /// the same start as MAXIMAL `match` would over the longest substring
    /// from there that it accepts.
    pub(crate) fn pos(&self, haystack: &[u8], minimal: bool) -> (usize, usize) {
        let Some((k, m)) = self.leftmost_shortest(haystack) else {
            return (0, 0);
        };
        if minimal {
            return (k + 1, k + m);
        }
        let rest = &haystack[k..];
        let mut longest = m;
        let mut before_nul = None;
        self.run(rest, |j| {
            longest = j;
            if j < rest.len() && rest[j] == 0 {
                before_nul = Some(j);
                return true;
            }
            false
        });
        match before_nul {
            Some(j) => (k + 1, k + j + 1),
            None => (k + 1, k + longest),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CompileError, ILLEGAL_NUMBER, STATE_BUDGET, compile};

    /// A count whose copies would pass the budget is error 5 at its `{`,
    /// found before anything is built; nested counts multiply.
    #[test]
    fn counts_past_the_budget_are_illegal_numbers() {
        for (pattern, open) in [
            ("a{2000000000}", 1),
            ("a{2147483647}", 1),
            ("((a{1000}){1000}){1000}", 17),
            ("x(a{50000}){50000}", 11),
            ("a{1500000}b{1500000}", 11),
        ] {
            assert_eq!(
                compile(pattern.as_bytes()).err(),
                Some(CompileError {
                    code: ILLEGAL_NUMBER,
                    position: open
                }),
                "{pattern}"
            );
        }
    }

    /// Counts within the budget compile and match as before, the copies
    /// sharing one byte set.
    #[test]
    fn counts_within_the_budget_compile() {
        let (regex, consumed) = compile(b"(a{1000}){1000}").expect("within the budget");
        assert_eq!(consumed, 15);
        assert_eq!(regex.sets.len(), 1);
        assert!(regex.states.len() as u64 <= STATE_BUDGET + 1);
        let subject = vec![b'a'; 1_000_000];
        assert_eq!(regex.match_maximal(&subject), (true, 1_000_000));
        assert!(!regex.match_maximal(&subject[1..]).0);
    }
}
