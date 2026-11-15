//! Photo mode's picture: the frame drawn once more without the interface, at the
//! display's resolution, read back, and saved as a PNG in the folder the settings name.
//! Nothing is uploaded anywhere.

use std::path::{Path, PathBuf};

use vellmark_gpu::{wgpu, Gpu, PassTimers};
use vellmark_render::{FrameInput, Renderer};

/// Where pictures go unless the settings name a folder: the user's pictures, in a folder
/// of the game's own.
pub fn default_folder() -> PathBuf {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join("Pictures").join("Vellmark")
}

/// Draws `frame` without its interface and saves it in `folder`. Returns the file.
pub fn save(gpu: &Gpu, renderer: &mut Renderer, frame: &FrameInput<'_>, size: (u32, u32), folder: &Path) -> Result<PathBuf, String> {
    let format = renderer.target_format();
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("picture"),
        size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let mut timers = PassTimers::new(gpu, false);
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    renderer.render(gpu, &mut encoder, &view, size, &FrameInput { ui: None, ..*frame }, &mut timers);
    gpu.queue.submit([encoder.finish()]);
    let mut rgba = vellmark_gpu::read_texture_rgba8(gpu, &texture);
    if matches!(format, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb) {
        for p in rgba.chunks_exact_mut(4) {
            p.swap(0, 2);
        }
    }
    for p in rgba.chunks_exact_mut(4) {
        p[3] = 255;
    }
    std::fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let mut path = folder.join(format!("vellmark-{stamp}.png"));
    let mut n = 1;
    while path.exists() {
        path = folder.join(format!("vellmark-{stamp}-{n}.png"));
        n += 1;
    }
    let file = std::fs::File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), size.0, size.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(&rgba).map_err(|e| e.to_string())?;
    Ok(path)
}
