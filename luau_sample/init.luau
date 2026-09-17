local MyWallpaper = {}
MyWallpaper.__index = MyWallpaper

function MyWallpaper.new(ctx: WallpaperInitContext): Wallpaper
    local self = {} :: Wallpaper

    return setmetatable(self, MyWallpaper)
end

function MyWallpaper:draw(ctx: WallpaperDrawContext)
    ctx:test()
end

local Factory: WallpaperFactory = {
    init = MyWallpaper.new,
}

return Factory
