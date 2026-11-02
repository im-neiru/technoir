///|
fn emit(level : @ffi.Level, target : String?, message : String) -> Unit {
  @ffi.event(level, target, message, None, [])
}

///|
/// Emits a trace-level logging event.
///
/// Use for highly detailed diagnostic information, typically useful
/// when investigating execution flow or internal state.
pub fn trace(target : String?, message : String) -> Unit {
  emit(@ffi.TRACE, target, message)
}

///|
/// Emits a debug-level logging event.
///
/// Use for diagnostic information intended to help developers
/// understand application behavior during development.
pub fn debug(target : String?, message : String) -> Unit {
  emit(@ffi.DEBUG, target, message)
}

///|
/// Emits an informational logging event.
///
/// Use for noteworthy application events and normal operational
/// messages that do not indicate a problem.
pub fn info(target : String?, message : String) -> Unit {
  emit(@ffi.INFO, target, message)
}

///|
/// Emits a warning-level logging event.
///
/// Use when something unexpected occurs but execution can continue
/// without immediate failure.
pub fn warn(target : String?, message : String) -> Unit {
  emit(@ffi.WARN, target, message)
}

///|
/// Emits an error-level logging event.
///
/// Use when an operation fails or an error condition prevents
/// the expected behavior of a component.
pub fn error(target : String?, message : String) -> Unit {
  emit(@ffi.ERROR, target, message)
}
