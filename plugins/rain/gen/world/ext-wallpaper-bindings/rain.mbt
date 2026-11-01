///|
pub fn prepare() -> Unit {
  @logcat.info("rain", "Wallpaper plugin prepared")
}

///|
pub fn draw() -> Unit {
  @logcat.debug("rain", "Drawing wallpaper")
}

///|
pub fn close() -> Unit {
  @logcat.info("rain", "Wallpaper plugin closed")
}
