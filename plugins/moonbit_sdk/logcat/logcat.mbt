fn emit(level : @noir_logcat.Level, target : String, message : String) -> Unit {
  @noir_logcat.event(level, target, message, None, [])
}

pub fn trace(target : String, message : String) -> Unit {
  emit(@noir_logcat.TRACE, target, message)
}

pub fn debug(target : String, message : String) -> Unit {
  emit(@noir_logcat.DEBUG, target, message)
}

pub fn info(target : String, message : String) -> Unit {
  emit(@noir_logcat.INFO, target, message)
}

pub fn warn(target : String, message : String) -> Unit {
  emit(@noir_logcat.WARN, target, message)
}

pub fn error(target : String, message : String) -> Unit {
  emit(@noir_logcat.ERROR, target, message)
}
