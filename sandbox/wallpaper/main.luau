local MyWallpaper = {}
MyWallpaper.__index = MyWallpaper

function MyWallpaper.new(ctx: WallpaperInitContext): Wallpaper
    local self = {} :: Wallpaper

    print("Hello rust from luau")

    return setmetatable(self, MyWallpaper)
end

function MyWallpaper:draw(ctx: WallpaperDrawContext)
    ctx:test()
    print("Hello lua")
end

return MyWallpaper.new
