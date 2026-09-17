use mlua::{Function, Table};

pub(crate) struct WallpaperProvider {
    table: Table,
    init: Function,
    draw: Function,
}
