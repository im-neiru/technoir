export type WallpaperInitContext = {
    -- todo
}

export type WallpaperDrawContext = {
    test: (self: WallpaperDrawContext) -> (),
}

export type Wallpaper = {
    draw: (self: Wallpaper, ctx: WallpaperDrawContext) -> (),
}

export type WallpaperFactory = (ctx: WallpaperInitContext) -> Wallpaper
