

local MyWallpaper = {}
MyWallpaper.__index = MyWallpaper

function MyWallpaper:draw(ctx: WallpaperDrawContext)
    ctx:test()
end
