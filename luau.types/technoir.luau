export type WallpaperInitContext = {
    -- todo
}

export type WallpaperDrawContext = {
    test: (WallpaperDrawContext) -> (),
}

export type Wallpaper = {
    draw: (self: Wallpaper, ctx: WallpaperDrawContext) -> (),
}

export type WallpaperProvider = {
    init: (ctx: WallpaperInitContext) -> Wallpaper,
}
