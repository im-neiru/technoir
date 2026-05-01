pub(super) trait WallpaperDriver {
    fn run(&mut self);

    fn terminate(&mut self);
}
