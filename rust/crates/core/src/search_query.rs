//! Port of `lib/search-query.mjs`: the client search syntax as an expression tree over score maps.

use std::sync::LazyLock;

use indexmap::IndexMap;
use mm_js::{
    Value,
    number::string_to_number,
    obj,
    text::{is_whitespace, len, to_lower, trim},
};
use regex::Regex;

use crate::render::access::prototype_member;
use crate::{Error, Result};

const TYPE_ALIASES: [(&str, &str); 24] = [
    ("XPost", "x-post post tweet"), ("WebPage", "web-page webpage website bookmark"),
    ("SoftwareApplication", "software-application app"), ("Content", "content snippet text"),
    ("Quotation", "quotation quote text"), ("Quote", "quotation quote text"), ("Note", "note text"),
    ("YouTubeVideo", "youtube-video video"), ("VimeoVideo", "vimeo-video video"),
    ("InstagramReel", "instagram-reel instagramreel video reel post"), ("InstagramPost", "instagram-post post"),
    ("ThreadsPost", "threads-post post"), ("BlueskyPost", "bluesky-post post"), ("SubstackNote", "substack-note post"),
    ("TikTokPost", "tiktok-post tiktok-video post video"), ("Embed", "embed video"), ("TedTalk", "ted-talk video"),
    ("RealProperty", "real-property real-estate house"), ("Palette", "palette color colour"),
    ("MusicRecording", "music-recording track song"), ("MusicAlbum", "music-album album"),
    ("MusicPlaylist", "music-playlist playlist"), ("VideoGame", "video-game game"), ("", ""),
];

static LOWER_UPPER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([a-z0-9])([A-Z])").expect("regex"));
static UPPER_RUN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([A-Z])([A-Z][a-z])").expect("regex"));
static BOOST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\^([0-9]+(?:\.[0-9]+)?)$").expect("regex"));
static QUOTE_TAIL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""(?:~[0-9]+)?$"#).expect("regex"));

/// Return the type slugs understood by the shipped client's canonicalizer and named queries.
///
/// # Arguments
/// * `kind` - Stored PascalCase entity type.
///
/// # Returns
/// Exact type and client aliases, including parent categories.
///
/// # Errors
/// `kind` names an `Object.prototype` member, which the JS alias lookup turns into a TypeError.
pub fn type_slugs(kind: &str) -> Result<Vec<String>> {
    if prototype_member(kind).is_some() {
        return Err(Error::invalid("(TYPE_ALIASES[type] ?? \"\").split is not a function"));
    }
    let spaced = LOWER_UPPER.replace_all(kind, "$1-$2");
    let slug = to_lower(&UPPER_RUN.replace_all(&spaced, "$1-$2"));
    let aliases = TYPE_ALIASES.iter().find(|(key, _)| *key == kind && !key.is_empty()).map_or("", |(_, value)| value);
    let mut out: Vec<String> = Vec::new();
    for alias in aliases.split(' ').filter(|alias| !alias.is_empty()).chain(std::iter::once(slug.as_str())) {
        if !out.iter().any(|known| known == alias) {
            out.push(alias.to_string());
        }
    }
    Ok(out)
}

fn tokens(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let pair = |at: usize| chars.get(at).zip(chars.get(at + 1)).is_some_and(|(a, b)| a == b && (*a == '&' || *a == '|'));
    let mut out = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        if is_whitespace(chars[at]) {
            at += 1;
            continue;
        }
        if pair(at) {
            out.push(chars[at..at + 2].iter().collect());
            at += 2;
            continue;
        }
        if "()-!+".contains(chars[at]) {
            out.push(chars[at].to_string());
            at += 1;
            continue;
        }
        let (mut word, mut quote, mut range) = (String::new(), false, false);
        while at < chars.len() {
            let character = chars[at];
            if character == '\\' && at + 1 < chars.len() {
                word.push(character);
                at += 1;
                word.push(chars[at]);
                at += 1;
                continue;
            }
            if character == '"' {
                quote = !quote;
            }
            if !quote && character == '[' {
                range = true;
            }
            if !quote && character == ']' {
                range = false;
            }
            if !quote && !range && (is_whitespace(character) || character == '(' || character == ')' || pair(at)) {
                break;
            }
            word.push(character);
            at += 1;
        }
        if !word.is_empty() {
            out.push(word);
        }
    }
    out
}

/// The length of a leading `[a-z]+:` (case-insensitive), or `None`.
fn field_prefix(raw: &str) -> Option<usize> {
    let letters = raw.bytes().take_while(u8::is_ascii_alphabetic).count();
    (letters > 0 && raw.as_bytes().get(letters) == Some(&b':')).then_some(letters + 1)
}

/// One decoded field/value term.
#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    pub field: String,
    pub value: String,
    pub quoted: bool,
    pub prefix: bool,
    pub boost: f64,
}

impl Term {
    /// The term as the JS node object.
    pub fn to_value(&self) -> Value {
        obj! {
            "kind" => "term", "field" => self.field.as_str(), "value" => self.value.as_str(), "quoted" => self.quoted,
            "prefix" => self.prefix, "boost" => self.boost
        }
        .into()
    }
}

fn atom(mut raw: &str) -> Term {
    let boost = BOOST.captures(raw);
    if let Some(found) = &boost {
        raw = &raw[..found.get(0).expect("match").start()];
    }
    let field_length = field_prefix(raw);
    let field = match field_length {
        Some(length) => raw[..length - 1].to_ascii_lowercase(),
        None if raw.starts_with('#') => "tag".into(),
        None => String::new(),
    };
    let skip = field_length.unwrap_or(usize::from(field == "tag"));
    let mut value = raw[skip..].to_string();
    let quoted = value.starts_with('"');
    let prefix = !quoted && value.ends_with('*');
    if prefix {
        value.pop();
    }
    if quoted {
        let rest = &value[1..];
        value = unescape(&QUOTE_TAIL.replace_all(rest, ""));
    }
    let boost = boost.map_or(1.0, |found| string_to_number(&found[1]));
    Term { field, value, quoted, prefix, boost: boost.min(100.0) }
}

/// `replace(/\\(.)/g, '$1')`.
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\\' && chars.peek().is_some_and(|next| !matches!(next, '\n' | '\r' | '\u{2028}' | '\u{2029}')) {
            out.push(chars.next().expect("peeked"));
        } else {
            out.push(character);
        }
    }
    out
}

/// A node of the parsed expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    All,
    Empty,
    /// An index into [`Parsed::terms`].
    Term(usize),
    Not(Box<Node>),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
}

/// The parsed query.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub tree: Node,
    pub terms: Vec<Term>,
    /// The normalized display query.
    pub query: String,
}

impl Parsed {
    /// The `parseQuery` result object, with term nodes inlined in the tree.
    pub fn to_value(&self) -> Value {
        obj! { "tree" => self.node_value(&self.tree), "terms" => Value::Array(self.terms.iter().map(Term::to_value).collect()), "query" => self.query.as_str() }.into()
    }

    fn node_value(&self, node: &Node) -> Value {
        match node {
            Node::All => obj! { "kind" => "all" }.into(),
            Node::Empty => obj! { "kind" => "empty" }.into(),
            Node::Term(index) => self.terms[*index].to_value(),
            Node::Not(child) => obj! { "kind" => "not", "child" => self.node_value(child) }.into(),
            Node::And(left, right) => obj! { "kind" => "and", "left" => self.node_value(left), "right" => self.node_value(right) }.into(),
            Node::Or(left, right) => obj! { "kind" => "or", "left" => self.node_value(left), "right" => self.node_value(right) }.into(),
        }
    }
}

struct Parser {
    parts: Vec<String>,
    at: usize,
    terms: Vec<Term>,
}

impl Parser {
    fn part(&self, at: usize) -> Option<&str> {
        self.parts.get(at).map(String::as_str)
    }

    fn is_or(part: Option<&str>) -> bool {
        matches!(part, Some("OR" | "||"))
    }

    fn is_and(part: Option<&str>) -> bool {
        matches!(part, Some("AND" | "&&"))
    }

    fn unary(&mut self) -> Node {
        let part = self.part(self.at).map(str::to_string);
        self.at += 1;
        let Some(part) = part else { return Node::Empty };
        if matches!(part.as_str(), "-" | "!" | "NOT") {
            return Node::Not(Box::new(self.unary()));
        }
        if part == "+" {
            return self.unary();
        }
        if part == "(" {
            let node = self.disjunction();
            if self.part(self.at) == Some(")") {
                self.at += 1;
            }
            return node;
        }
        if part == ")" {
            return Node::Empty;
        }
        let mut raw = part;
        let lowered = raw.to_ascii_lowercase();
        let tagged = ["tag:", "label:"].iter().any(|prefix| lowered.starts_with(prefix) && raw[prefix.len()..].chars().next().is_some_and(|c| c != '"' && c != '['));
        if tagged || raw.starts_with('#') {
            while let Some(next) = self.part(self.at) {
                if Self::is_or(Some(next)) || Self::is_and(Some(next)) || matches!(next, "(" | ")" | "-" | "!" | "+" | "NOT")
                    || field_prefix(next).is_some() || next.starts_with('#') || next.starts_with('"')
                {
                    break;
                }
                raw.push(' ');
                raw.push_str(next);
                self.at += 1;
            }
        }
        if field_prefix(&raw) == Some(raw.len())
            && let Some(next) = self.part(self.at).filter(|next| !matches!(*next, "(" | ")" | "-"))
        {
            raw.push_str(next);
            self.at += 1;
        }
        self.terms.push(atom(&raw));
        Node::Term(self.terms.len() - 1)
    }

    fn conjunction(&mut self) -> Node {
        let mut node = self.unary();
        while self.at < self.parts.len() && self.part(self.at) != Some(")") && !Self::is_or(self.part(self.at)) {
            if Self::is_and(self.part(self.at)) {
                self.at += 1;
            }
            node = Node::And(Box::new(node), Box::new(self.unary()));
        }
        node
    }

    fn disjunction(&mut self) -> Node {
        let mut node = self.conjunction();
        while Self::is_or(self.part(self.at)) {
            self.at += 1;
            node = Node::Or(Box::new(node), Box::new(self.conjunction()));
        }
        node
    }
}

/// Parse client query syntax into an expression tree with AND precedence over OR.
///
/// # Arguments
/// * `query` - User text; limited to 8,192 characters and 256 terms.
/// * `wildcard` - Prefix-match the final unquoted free-text term.
///
/// # Returns
/// The tree, its safe terms (tree nodes index into them) and the normalized display query.
///
/// # Errors
/// Status 400 when the query is longer than 8,192 UTF-16 units or has more than 256 tokens.
pub fn parse_query(query: &str, wildcard: bool) -> Result<Parsed> {
    let text: String = trim(query).chars().map(|c| if "\u{201c}\u{201d}\u{ab}\u{bb}".contains(c) { '"' } else { c }).collect();
    if len(&text) > 8192 {
        return Err(Error::status(400, "Search query is too long"));
    }
    let parts = tokens(&text);
    if parts.len() > 256 {
        return Err(Error::status(400, "Search query has too many terms"));
    }
    let empty = parts.is_empty();
    let mut parser = Parser { parts, at: 0, terms: Vec::new() };
    let tree = if empty { Node::All } else { parser.disjunction() };
    let mut terms = parser.terms;
    let append = wildcard && terms.last().is_some_and(|last| last.field.is_empty() && !last.quoted && !last.prefix) && !text.ends_with(')');
    if append {
        terms.last_mut().expect("last term").prefix = true;
    }
    let query = format!("{text}{}", if append { "*" } else { "" });
    Ok(Parsed { tree, terms, query })
}

/// Matching card IDs with their scores, in insertion order.
pub type Scores = IndexMap<String, f64>;

/// Evaluate a parsed expression using score maps supplied by the search engine.
///
/// # Arguments
/// * `node` - Parsed expression node.
/// * `terms` - The terms the node's `Term` indices refer to.
/// * `universe` - Visible cards with a neutral score.
/// * `resolve` - Resolves one term, given whether it sits under an odd number of negations.
/// * `negative` - Whether positive text is under an odd number of negations.
///
/// # Returns
/// Matching IDs with combined scores.
///
/// # Errors
/// Whatever `resolve` returns.
pub fn evaluate(
    node: &Node,
    terms: &[Term],
    universe: &Scores,
    resolve: &mut dyn FnMut(&Term, bool) -> Result<Scores>,
    negative: bool,
) -> Result<Scores> {
    Ok(match node {
        Node::All => universe.clone(),
        Node::Empty => Scores::new(),
        Node::Term(index) => resolve(&terms[*index], negative)?,
        Node::Not(child) => {
            let excluded = evaluate(child, terms, universe, resolve, !negative)?;
            universe.iter().filter(|(id, _)| !excluded.contains_key(*id)).map(|(id, score)| (id.clone(), *score)).collect()
        }
        Node::And(left, right) => {
            let left = evaluate(left, terms, universe, resolve, negative)?;
            let right = evaluate(right, terms, universe, resolve, negative)?;
            left.iter().filter_map(|(id, score)| right.get(id).map(|other| (id.clone(), score + other))).collect()
        }
        Node::Or(left, right) => {
            let mut left = evaluate(left, terms, universe, resolve, negative)?;
            let right = evaluate(right, terms, universe, resolve, negative)?;
            for (id, score) in right {
                let total = left.get(&id).copied().unwrap_or(0.0) + score;
                left.insert(id, total);
            }
            left
        }
    })
}
