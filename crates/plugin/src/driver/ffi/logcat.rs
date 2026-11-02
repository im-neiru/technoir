use wasmtime::{Caller, Linker, Memory};

use super::HostState;

const IMPORT_MODULE: &str = "technoir:plugin/logcat@0.1.0";

pub(super) fn add_to_linker(linker: &mut Linker<HostState>) -> wasmtime::Result<()> {
    linker.func_wrap(IMPORT_MODULE, "enabled", enabled)?;
    linker.func_wrap(IMPORT_MODULE, "event", event)?;
    linker.func_wrap(IMPORT_MODULE, "span-start", span_start)?;
    linker.func_wrap(IMPORT_MODULE, "span-end", span_end)?;

    Ok(())
}

fn enabled(_level: i32, _target_present: i32, _target_pointer: i32, _target_length: i32) -> i32 {
    1
}

fn event(
    mut caller: Caller<'_, HostState>,
    level: i32,
    target_present: i32,
    target_pointer: i32,
    target_length: i32,
    message_pointer: i32,
    message_length: i32,
    _span_present: i32,
    _span: i64,
    _fields_pointer: i32,
    _fields_length: i32,
) -> wasmtime::Result<()> {
    let target =
        read_optional_guest_string(&mut caller, target_present, target_pointer, target_length)?;

    let message = read_guest_string(&mut caller, message_pointer, message_length)?;
    let level = log_level_name(level)?;
    let line = format_log_event(level, target.as_deref(), &message);

    println!("{line}");
    caller.data_mut().logs.push(line);

    Ok(())
}

fn span_start(
    mut caller: Caller<'_, HostState>,
    _level: i32,
    _target_present: i32,
    _target_pointer: i32,
    _target_length: i32,
    _name_pointer: i32,
    _name_length: i32,
    _parent_present: i32,
    _parent: i64,
    _fields_pointer: i32,
    _fields_length: i32,
    return_pointer: i32,
) -> wasmtime::Result<()> {
    let return_pointer = usize::try_from(return_pointer)
        .map_err(|_| wasmtime::format_err!("guest return pointer is negative"))?;

    let memory = guest_memory(&mut caller)?;
    memory.write(&mut caller, return_pointer, &[0])?;

    Ok(())
}

fn span_end(_id: i64) {}

fn read_optional_guest_string(
    caller: &mut Caller<'_, HostState>,
    is_present: i32,
    pointer: i32,
    length: i32,
) -> wasmtime::Result<Option<String>> {
    if is_present == 0 {
        return Ok(None);
    }

    let value = read_guest_string(caller, pointer, length)?;

    Ok(Some(value))
}

fn read_guest_string(
    caller: &mut Caller<'_, HostState>,
    pointer: i32,
    length: i32,
) -> wasmtime::Result<String> {
    let pointer = usize::try_from(pointer)
        .map_err(|_| wasmtime::format_err!("guest string pointer is negative"))?;

    let length = usize::try_from(length)
        .map_err(|_| wasmtime::format_err!("guest string length is negative"))?;

    let byte_length = length
        .checked_mul(2)
        .ok_or_else(|| wasmtime::format_err!("guest string length overflows"))?;

    let memory = guest_memory(caller)?;
    let mut bytes = vec![0; byte_length];

    memory.read(&*caller, pointer, &mut bytes)?;

    let code_units = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]));
    let code_units = code_units.collect::<Vec<_>>();

    String::from_utf16(&code_units).map_err(wasmtime::Error::new)
}

fn log_level_name(level: i32) -> wasmtime::Result<&'static str> {
    match level {
        0 => Ok("Trace"),
        1 => Ok("Debug"),
        2 => Ok("Info"),
        3 => Ok("Warn"),
        4 => Ok("Error"),
        _ => Err(wasmtime::format_err!("unknown log level {level}")),
    }
}

fn format_log_event(level: &str, target: Option<&str>, message: &str) -> String {
    match target {
        Some(target) => format!("[Level::{level}] {target}: {message}"),
        None => format!("[Level::{level}] {message}"),
    }
}

fn guest_memory(caller: &mut Caller<'_, HostState>) -> wasmtime::Result<Memory> {
    caller
        .get_export("memory")
        .and_then(|export| export.into_memory())
        .ok_or_else(|| wasmtime::format_err!("guest module does not export memory"))
}
