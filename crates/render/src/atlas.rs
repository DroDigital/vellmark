use vellmark_content::GlyphAtlas;
use vellmark_gpu::{Gpu, wgpu};

/// The glyph atlas on the GPU: one array texture with a layer for each character of
/// each typeface, shared by every surface and by the interface.
pub struct AtlasGpu {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub cell: u32,
    pub range: f32,
}

impl AtlasGpu {
    pub fn new(gpu: &Gpu, atlas: &GlyphAtlas) -> Self {
        let layers = atlas.layers();
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph atlas"),
            size: wgpu::Extent3d {
                width: atlas.cell,
                height: atlas.cell,
                depth_or_array_layers: layers,
            },
            mip_level_count: atlas.mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for level in 0..atlas.mips {
            let side = atlas.cell >> level;
            gpu.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: level,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &atlas.texels[atlas.level_range(level)],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(side * 4),
                    rows_per_image: Some(side),
                },
                wgpu::Extent3d {
                    width: side,
                    height: side,
                    depth_or_array_layers: layers,
                },
            );
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("glyph atlas"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        Self {
            view,
            sampler,
            cell: atlas.cell,
            range: atlas.range,
        }
    }
}
