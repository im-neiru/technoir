use rapidhash::RapidHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Pipelines(pub RapidHashMap<String, Pipeline>);

impl Pipelines {
    pub fn get(&self, name: &str) -> Option<&Pipeline> {
        self.0.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Pipeline)> {
        self.0.iter()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pipeline {
    /// Shader path relative to the plugin's `shaders/` directory.
    pub shader: String,

    /// Explicit entry points.
    ///
    /// `None` means the engine asks Naga to select the unique entry point.
    #[serde(default)]
    pub entry_points: Option<EntryPoints>,

    /// Pipeline-overridable shader constants.
    ///
    /// Empty when omitted.
    #[serde(default)]
    pub constants: RapidHashMap<String, f64>,

    /// Actual vertex-buffer layouts.
    ///
    /// Explicitly defaults to no vertex buffers.
    #[serde(default)]
    pub vertex_buffers: Vec<VertexBuffer>,

    /// Color target state.
    ///
    /// Explicitly defaults to one color target using opaque rendering.
    #[serde(default = "default_targets")]
    pub targets: Vec<Option<ColorTarget>>,

    /// Primitive assembly/rasterization state.
    #[serde(default)]
    pub primitive: PrimitiveState,

    /// Depth/stencil state.
    ///
    /// `None` means the pipeline does not use a depth/stencil attachment.
    #[serde(default)]
    pub depth_stencil: Option<DepthStencilState>,

    /// Multisampling state.
    #[serde(default)]
    pub multisample: MultisampleState,
}

fn default_targets() -> Vec<Option<ColorTarget>> {
    vec![Some(ColorTarget::default())]
}

// -----------------------------------------------------------------------------
// Entry points
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryPoints {
    #[serde(default)]
    pub vertex: Option<String>,

    #[serde(default)]
    pub fragment: Option<String>,
}

// -----------------------------------------------------------------------------
// Vertex buffers
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct VertexBuffer {
    /// Vertex-buffer slot.
    pub slot: u32,

    /// Size in bytes of one vertex/instance record.
    pub array_stride: u64,

    /// Whether the buffer advances per vertex or per instance.
    #[serde(default)]
    pub step_mode: StepMode,

    /// Attributes contained in this buffer.
    pub attributes: Vec<VertexAttribute>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum StepMode {
    #[default]
    Vertex,

    Instance,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VertexAttribute {
    /// Physical format in GPU memory.
    pub format: VertexFormat,

    /// Byte offset within one vertex/instance record.
    pub offset: u64,

    /// Shader input location.
    pub shader_location: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum VertexFormat {
    Uint8,
    Uint8x2,
    Uint8x4,

    Sint8,
    Sint8x2,
    Sint8x4,

    Unorm8,
    Unorm8x2,
    Unorm8x4,

    Snorm8,
    Snorm8x2,
    Snorm8x4,

    Uint16,
    Uint16x2,
    Uint16x4,

    Sint16,
    Sint16x2,
    Sint16x4,

    Unorm16,
    Unorm16x2,
    Unorm16x4,

    Snorm16,
    Snorm16x2,
    Snorm16x4,

    Float16,
    Float16x2,
    Float16x4,

    Float32,
    Float32x2,
    Float32x3,
    Float32x4,

    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,

    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,

    Unorm1010102,
    Unorm8x4Bgra,
}

// -----------------------------------------------------------------------------
// Color targets
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ColorTarget {
    /// Blend behavior.
    #[serde(default)]
    pub blend: BlendState,

    /// Channels written by the fragment shader.
    #[serde(default)]
    pub write_mask: WriteMask,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BlendState {
    Preset(BlendPreset),
    Custom(CustomBlendState),
}

impl Default for BlendState {
    fn default() -> Self {
        Self::Preset(BlendPreset::Opaque)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum BlendPreset {
    /// Blending disabled; source replaces destination.
    #[default]
    Opaque,

    /// Conventional straight-alpha blending.
    Alpha,

    /// Premultiplied-alpha blending.
    PremultipliedAlpha,

    /// Add source to destination.
    Additive,

    /// Explicit replacement behavior.
    Replace,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomBlendState {
    pub color: BlendComponent,
    pub alpha: BlendComponent,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlendComponent {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub operation: BlendOperation,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum BlendFactor {
    Zero,
    One,
    Src,
    OneMinusSrc,
    SrcAlpha,
    OneMinusSrcAlpha,
    Dst,
    OneMinusDst,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturated,
    Constant,
    OneMinusConstant,
    Src1,
    OneMinusSrc1,
    Src1Alpha,
    OneMinusSrc1Alpha,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum BlendOperation {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

#[derive(Debug, Default)]
pub enum WriteMask {
    #[default]
    All,
    None,
    Channels(Vec<ColorChannel>),
}

impl<'de> Deserialize<'de> for WriteMask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Preset(String),
            Channels(Vec<ColorChannel>),
        }

        match Repr::deserialize(deserializer)? {
            Repr::Preset(value) => match value.as_str() {
                "All" => Ok(Self::All),
                "None" => Ok(Self::None),
                _ => Err(serde::de::Error::unknown_variant(&value, &["All", "None"])),
            },
            Repr::Channels(channels) => Ok(Self::Channels(channels)),
        }
    }
}

impl Serialize for WriteMask {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::All => serializer.serialize_str("All"),
            Self::None => serializer.serialize_str("None"),
            Self::Channels(channels) => channels.serialize(serializer),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
    Alpha,
}

// -----------------------------------------------------------------------------
// Primitive state
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PrimitiveState {
    /// How vertices are assembled.
    ///
    /// Explicit default: TriangleList.
    #[serde(default)]
    pub topology: PrimitiveTopology,

    /// Required only for indexed strip primitives.
    #[serde(default)]
    pub strip_index_format: Option<IndexFormat>,

    /// Front-facing winding order.
    ///
    /// Explicit default: Ccw.
    #[serde(default)]
    pub front_face: FrontFace,

    /// Faces discarded before fragment shading.
    ///
    /// Explicit default: None.
    #[serde(default)]
    pub cull_mode: CullMode,

    /// Allow depth outside the normal clip range.
    ///
    /// Explicit default: false.
    #[serde(default)]
    pub unclipped_depth: bool,

    /// Polygon rasterization mode.
    ///
    /// Explicit default: Fill.
    #[serde(default)]
    pub polygon_mode: PolygonMode,

    /// Conservative rasterization.
    ///
    /// Explicit default: false.
    #[serde(default)]
    pub conservative: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,

    #[default]
    TriangleList,

    TriangleStrip,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum IndexFormat {
    Uint16,
    Uint32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum FrontFace {
    #[default]
    Ccw,

    Cw,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum CullMode {
    #[default]
    None,

    Front,
    Back,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum PolygonMode {
    #[default]
    Fill,

    Line,
    Point,
}

// -----------------------------------------------------------------------------
// Depth / stencil
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DepthStencilState {
    /// Depth/stencil attachment format.
    ///
    /// Explicit default: Depth32Float.
    #[serde(default)]
    pub format: TextureFormat,

    /// Depth-test state.
    ///
    /// `None` means depth testing is disabled even though this is a
    /// depth/stencil pipeline.
    #[serde(default)]
    pub depth: Option<DepthState>,

    /// Stencil state.
    #[serde(default)]
    pub stencil: Option<StencilState>,

    /// Depth bias.
    ///
    /// Omitted means a zero bias.
    #[serde(default)]
    pub bias: DepthBiasState,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum TextureFormat {
    Stencil8,

    Depth16Unorm,
    Depth24Plus,
    Depth24PlusStencil8,

    #[default]
    Depth32Float,

    Depth32FloatStencil8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DepthState {
    /// Whether passing fragments update the depth buffer.
    #[serde(default = "default_depth_write_enabled")]
    pub write_enabled: bool,

    /// Depth comparison.
    #[serde(default)]
    pub compare: CompareFunction,
}

fn default_depth_write_enabled() -> bool {
    true
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            write_enabled: true,
            compare: CompareFunction::default(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum CompareFunction {
    Never,
    Less,
    Equal,

    #[default]
    LessEqual,

    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

// -----------------------------------------------------------------------------
// Stencil
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct StencilState {
    /// State for front-facing primitives.
    #[serde(default)]
    pub front: StencilFaceState,

    /// State for back-facing primitives.
    #[serde(default)]
    pub back: StencilFaceState,

    /// Bits read by stencil operations.
    #[serde(default = "default_stencil_mask")]
    pub read_mask: u32,

    /// Bits written by stencil operations.
    #[serde(default = "default_stencil_mask")]
    pub write_mask: u32,
}

fn default_stencil_mask() -> u32 {
    0xFF
}

impl Default for StencilState {
    fn default() -> Self {
        Self {
            front: StencilFaceState::default(),
            back: StencilFaceState::default(),
            read_mask: 0xFF,
            write_mask: 0xFF,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StencilFaceState {
    #[serde(default)]
    pub compare: CompareFunction,

    #[serde(default)]
    pub fail_op: StencilOperation,

    #[serde(default)]
    pub depth_fail_op: StencilOperation,

    #[serde(default)]
    pub pass_op: StencilOperation,
}

impl Default for StencilFaceState {
    fn default() -> Self {
        Self {
            compare: CompareFunction::Always,
            fail_op: StencilOperation::Keep,
            depth_fail_op: StencilOperation::Keep,
            pass_op: StencilOperation::Keep,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum StencilOperation {
    #[default]
    Keep,

    Zero,
    Replace,
    Invert,
    IncrementClamp,
    DecrementClamp,
    IncrementWrap,
    DecrementWrap,
}

// -----------------------------------------------------------------------------
// Depth bias
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct DepthBiasState {
    /// Constant depth bias.
    #[serde(default)]
    pub constant: i32,

    /// Slope-scaled depth bias.
    #[serde(default)]
    pub slope_scale: f32,

    /// Maximum absolute depth bias.
    #[serde(default)]
    pub clamp: f32,
}

impl Default for DepthBiasState {
    fn default() -> Self {
        Self {
            constant: 0,
            slope_scale: 0.0,
            clamp: 0.0,
        }
    }
}

// -----------------------------------------------------------------------------
// Multisampling
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct MultisampleState {
    /// Number of samples per pixel.
    ///
    /// Explicit default: 1.
    #[serde(default = "default_sample_count")]
    pub count: u32,

    /// Sample coverage mask.
    ///
    /// Explicit default: all samples enabled.
    #[serde(default = "default_sample_mask")]
    pub mask: u64,

    /// Alpha-to-coverage.
    ///
    /// Explicit default: false.
    #[serde(default)]
    pub alpha_to_coverage_enabled: bool,
}

#[inline]
const fn default_sample_count() -> u32 {
    1
}

#[inline]
const fn default_sample_mask() -> u64 {
    u64::MAX
}

impl Default for MultisampleState {
    fn default() -> Self {
        Self {
            count: 1,
            mask: u64::MAX,
            alpha_to_coverage_enabled: false,
        }
    }
}
