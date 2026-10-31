use core::fmt;

use naga::{
    WithSpan, back, front,
    valid::{self, Capabilities, ValidationFlags},
};
use rapidhash::RapidHashSet;

#[derive(Debug)]
pub(super) enum Error {
    Parse(front::wgsl::ParseError),
    Validate(Box<WithSpan<valid::ValidationError>>),
    Write(back::wgsl::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "WGSL parse error: {error}"),
            Self::Validate(error) => write!(f, "WGSL validation error: {error}"),
            Self::Write(error) => write!(f, "WGSL write error: {error}"),
        }
    }
}

impl std::error::Error for Error {}

pub(super) fn minify_wgsl(input: &str) -> Result<String, Error> {
    let mut module = front::wgsl::parse_str(input).map_err(Error::Parse)?;

    minify_names(&mut module);

    let info = valid::Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .map_err(|err| Error::Validate(Box::new(err)))?;

    let output = back::wgsl::write_string(&module, &info, back::wgsl::WriterFlags::empty())
        .map_err(Error::Write)?;

    Ok(minify_whitespace(&output))
}

fn minify_names(module: &mut naga::Module) {
    let mut global_used = RapidHashSet::default();

    for entry_point in &module.entry_points {
        global_used.insert(entry_point.name.clone());
    }
    for (_, override_) in module.overrides.iter() {
        if let Some(name) = &override_.name {
            global_used.insert(name.clone());
        }
    }
    for (_, ty) in module.types.iter() {
        if let Some(name) = &ty.name {
            global_used.insert(name.clone());
        }
    }

    let mut global_counter = 0;

    let mut global_options: Vec<&mut Option<String>> = Vec::new();
    for (_, constant) in module.constants.iter_mut() {
        global_options.push(&mut constant.name);
    }
    for (_, variable) in module.global_variables.iter_mut() {
        global_options.push(&mut variable.name);
    }
    for (_, function) in module.functions.iter_mut() {
        global_options.push(&mut function.name);
    }

    minify_optional_names(&mut global_options, &mut global_used, &mut global_counter);

    for (_, function) in module.functions.iter_mut() {
        minify_function(function, &global_used);
    }

    for entry_point in &mut module.entry_points {
        minify_function(&mut entry_point.function, &global_used);
    }
}

fn minify_function(function: &mut naga::Function, global_used: &RapidHashSet<String>) {
    let mut local_used = global_used.clone();
    let mut local_counter = 0;

    let mut local_options: Vec<&mut Option<String>> = Vec::new();
    for argument in function.arguments.iter_mut() {
        local_options.push(&mut argument.name);
    }
    for (_, variable) in function.local_variables.iter_mut() {
        local_options.push(&mut variable.name);
    }
    minify_optional_names(&mut local_options, &mut local_used, &mut local_counter);

    let mut named_expressions: Vec<&mut String> = function.named_expressions.values_mut().collect();

    minify_strings(&mut named_expressions, &mut local_used, &mut local_counter);
}

fn minify_optional_names(
    names: &mut [&mut Option<String>],
    used: &mut RapidHashSet<String>,
    counter: &mut usize,
) {
    for name_opt in names.iter() {
        if let Some(name) = name_opt.as_deref()
            && name.len() <= 1
        {
            used.insert(name.to_owned());
        }
    }

    names.sort_unstable_by(|a, b| {
        let a_str = a.as_deref().unwrap_or("");
        let b_str = b.as_deref().unwrap_or("");
        b_str.len().cmp(&a_str.len()).then_with(|| a_str.cmp(b_str))
    });

    for name_opt in names.iter_mut() {
        let Some(old) = name_opt.as_deref() else {
            continue;
        };
        if old.len() <= 1 {
            continue;
        }

        let new_name = next_valid_name(used, counter);
        **name_opt = Some(new_name);
    }
}

fn minify_strings(names: &mut [&mut String], used: &mut RapidHashSet<String>, counter: &mut usize) {
    for name in names.iter() {
        if name.len() <= 1 {
            used.insert((*name).clone());
        }
    }

    names.sort_unstable_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));

    for name in names.iter_mut() {
        if name.len() <= 1 {
            continue;
        }

        let new_name = next_valid_name(used, counter);
        **name = new_name;
    }
}

fn next_valid_name(used: &mut RapidHashSet<String>, counter: &mut usize) -> String {
    let mut buf = [0u8; 16];
    loop {
        let candidate_str = write_short_name(*counter, &mut buf);
        *counter += 1;

        if !used.contains(candidate_str)
            && !naga::keywords::wgsl::RESERVED_SET.contains(candidate_str)
            && !naga::keywords::wgsl::BUILTIN_IDENTIFIER_SET.contains(candidate_str)
        {
            let name = candidate_str.to_string();
            used.insert(name.clone());
            return name;
        }
    }
}

fn write_short_name(mut i: usize, buf: &mut [u8; 16]) -> &str {
    const FIRST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const NEXT: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    let mut len = 0;
    buf[0] = FIRST[i % FIRST.len()];
    len += 1;
    i /= FIRST.len();

    while i > 0 {
        i -= 1;
        buf[len] = NEXT[i % NEXT.len()];
        len += 1;
        i /= NEXT.len();
    }

    // SAFETY: Input slices contain ASCII characters only.
    unsafe { core::str::from_utf8_unchecked(&buf[..len]) }
}

fn minify_whitespace(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let bytes = input.as_bytes();

    let mut i = 0;
    let mut previous_non_space = None;

    while i < bytes.len() {
        let byte = bytes[i];

        if byte.is_ascii_whitespace() {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            if let Some(next) = bytes.get(i).copied()
                && let Some(previous) = previous_non_space
                && needs_separator(previous, next)
            {
                output.push(' ');
            }
            continue;
        }

        output.push(byte as char);
        previous_non_space = Some(byte);
        i += 1;
    }

    output
}

#[inline]
fn needs_separator(previous: u8, next: u8) -> bool {
    if is_word_byte(previous) && is_word_byte(next) {
        return true;
    }

    if is_operator_byte(previous) && is_operator_byte(next) {
        return true;
    }

    false
}

#[inline]
fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[inline]
fn is_operator_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'!' | b'%' | b'&' | b'*' | b'+' | b'-' | b'/' | b'<' | b'=' | b'>' | b'|' | b'^'
    )
}
