//! `node:util` `parseArgs` in strict mode, as the operational tools call it.

/// What an option takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// `type: 'string'`: the next argument or an inline `=value`.
    Text,
    /// `type: 'boolean'`: no value.
    Flag,
}

/// One declared option (`options.<name>`).
#[derive(Clone, Copy, Debug)]
pub struct Opt {
    /// Long name without the dashes.
    pub name: &'static str,
    /// Whether it takes a value.
    pub kind: Kind,
}

/// What `parseArgs` returned: option occurrences in command-line order and the positionals.
#[derive(Debug, Default)]
pub struct Parsed {
    options: Vec<(String, Option<String>)>,
    /// The positional arguments.
    pub positionals: Vec<String>,
}

impl Parsed {
    /// The last value given for a string option (`values[name]` without `multiple`).
    ///
    /// # Arguments
    /// * `name` - Option name.
    pub fn text(&self, name: &str) -> Option<&str> {
        self.options.iter().rev().find(|(option, _)| option == name).and_then(|(_, value)| value.as_deref())
    }

    /// Every value given for a string option (`values[name]` with `multiple: true`).
    ///
    /// # Arguments
    /// * `name` - Option name.
    pub fn texts(&self, name: &str) -> Vec<&str> {
        self.options.iter().filter(|(option, _)| option == name).filter_map(|(_, value)| value.as_deref()).collect()
    }

    /// Whether a boolean option was given.
    ///
    /// # Arguments
    /// * `name` - Option name.
    pub fn flag(&self, name: &str) -> bool {
        self.options.iter().any(|(option, _)| option == name)
    }
}

/// Parse arguments the way `parseArgs({ options, allowPositionals, strict: true })` does.
///
/// # Arguments
/// * `args` - The arguments after the program name.
/// * `declared` - The options the tool declares.
/// * `allow_positionals` - Whether positional arguments are accepted.
///
/// # Errors
/// The message of the `TypeError` `parseArgs` throws for an unknown option, a missing, ambiguous or unwanted value, or a
/// positional that is not allowed.
pub fn parse(args: &[String], declared: &[Opt], allow_positionals: bool) -> Result<Parsed, String> {
    let mut parsed = Parsed::default();
    let mut at = 0;
    while at < args.len() {
        let arg = &args[at];
        at += 1;
        if arg == "--" {
            for rest in &args[at..] {
                positional(&mut parsed, rest, allow_positionals)?;
            }
            break;
        }
        if arg.len() > 2 && arg.starts_with("--") {
            let (name, inline) = match arg[2..].split_once('=') {
                Some((name, value)) => (name, Some(value.to_owned())),
                None => (&arg[2..], None),
            };
            let Some(option) = declared.iter().find(|option| option.name == name) else {
                return Err(format!("TypeError [ERR_PARSE_ARGS_UNKNOWN_OPTION]: Unknown option '--{name}'"));
            };
            match (option.kind, inline) {
                (Kind::Flag, Some(_)) => return Err(format!("TypeError [ERR_PARSE_ARGS_INVALID_OPTION_VALUE]: Option '--{name}' does not take an argument")),
                (Kind::Flag, None) => parsed.options.push((name.to_owned(), None)),
                (Kind::Text, Some(value)) => parsed.options.push((name.to_owned(), Some(value))),
                (Kind::Text, None) => {
                    let Some(value) = args.get(at) else {
                        return Err(format!("TypeError [ERR_PARSE_ARGS_INVALID_OPTION_VALUE]: Option '--{name} <value>' argument missing"));
                    };
                    at += 1;
                    if value.len() > 1 && value.starts_with('-') {
                        return Err(format!("TypeError [ERR_PARSE_ARGS_INVALID_OPTION_VALUE]: Option '--{name}' argument is ambiguous.\nDid you forget to specify the option argument for '--{name}'?\nTo specify an option argument starting with a dash use '--{name}=-XYZ'."));
                    }
                    parsed.options.push((name.to_owned(), Some(value.clone())));
                }
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            return Err(format!("TypeError [ERR_PARSE_ARGS_UNKNOWN_OPTION]: Unknown option '{}'", &arg[..arg.chars().take(2).map(char::len_utf8).sum()]));
        } else {
            positional(&mut parsed, arg, allow_positionals)?;
        }
    }
    Ok(parsed)
}

fn positional(parsed: &mut Parsed, arg: &str, allowed: bool) -> Result<(), String> {
    if !allowed {
        return Err(format!("TypeError [ERR_PARSE_ARGS_UNEXPECTED_POSITIONAL]: Unexpected argument '{arg}'. This command does not take positional arguments"));
    }
    parsed.positionals.push(arg.to_owned());
    Ok(())
}
