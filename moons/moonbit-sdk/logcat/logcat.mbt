///|
fn emit(level : @ffi.Level, target : String, message : String) -> Unit {
  @ffi.event(level, target, message, None, [])
}

///|
pub fn trace(target : String, message : String) -> Unit {
  emit(@ffi.TRACE, target, message)
}

///|
pub fn debug(target : String, message : String) -> Unit {
  emit(@ffi.DEBUG, target, message)
}

///|
pub fn info(target : String, message : String) -> Unit {
  emit(@ffi.INFO, target, message)
}

///|
pub fn warn(target : String, message : String) -> Unit {
  emit(@ffi.WARN, target, message)
}

///|
pub fn error(target : String, message : String) -> Unit {
  emit(@ffi.ERROR, target, message)
}
