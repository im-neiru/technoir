local MyWallpaper = {}
MyWallpaper.__index = MyWallpaper

export type MyWallpaper = typeof(setmetatable(
	{} :: {
		mainPass: RenderPipeline,
		binding: BindingGroup,
		uTemporals: UniformBuffer,
		uScaling: UniformBuffer,
	},
	MyWallpaper
))

function MyWallpaper.new(ctx: WallpaperInitContext): MyWallpaper
	local mainPass = ctx.pipelines.mainPass
	local texBackground = ctx.inputs.background.asSampler()

	local uTemporals = mainPass.uniforms.temporals.createBuffer()
	local uScaling = mainPass.uniforms.scaling.createBuffer()
	local binding = mainPass.createBinding({ texBackground, uTemporals, uScaling })

	local self = {
		mainPass = mainPass,
		binding = binding,
		uTemporals = uTemporals,
		uScaling = uScaling,
	}

	return setmetatable(self, MyWallpaper)
end

function MyWallpaper:draw(ctx: WallpaperDrawContext)
	local pass = ctx.beginRender(ctx.screen)

	pass.setPipeline(self.mainPass)
	pass.setBindingGroup(0, self.binding, {})
	pass.draw()
end

return MyWallpaper
