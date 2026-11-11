use std::fmt;
use wgpu::rwh::{RawDisplayHandle, RawWindowHandle};

/// Why the game cannot draw. The application turns each of these into a readable
/// message from the string table; the details go to the log.
#[derive(Debug)]
pub enum GpuError {
    /// No graphics card or driver that speaks Vulkan, Metal, or DirectX 12.
    NoAdapter(String),
    /// A card was found, but it lacks a limit or feature the renderer needs.
    TooOld(String),
    /// The card refused to give a device or a surface.
    Failed(String),
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdapter(d) => write!(f, "no usable graphics adapter: {d}"),
            Self::TooOld(d) => write!(f, "graphics adapter lacks a requirement: {d}"),
            Self::Failed(d) => write!(f, "graphics device failed: {d}"),
        }
    }
}

impl std::error::Error for GpuError {}

#[derive(Clone, Debug, Default)]
pub struct GpuOptions {
    /// Ask for a software renderer. Used for captures in cloud sessions and CI, which
    /// have no real graphics card. Never used for a performance number.
    pub software: bool,
    /// Windows only: use Vulkan in place of DirectX 12.
    pub vulkan_on_windows: bool,
}

/// The raw handles of the window to draw into. The platform layer provides them.
#[derive(Clone, Copy)]
pub struct WindowHandles {
    pub window: RawWindowHandle,
    pub display: RawDisplayHandle,
}

pub struct Gpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub info: wgpu::AdapterInfo,
    /// Whether the GPU can time passes. When it cannot, pass timers report nothing.
    pub timestamps: bool,
}

/// Limits the renderer relies on. These are the WebGPU defaults, which all three
/// backends offer on any card the game supports.
fn required_limits() -> wgpu::Limits {
    wgpu::Limits::default()
}

fn backends(options: &GpuOptions) -> wgpu::Backends {
    if cfg!(target_os = "windows") {
        if options.vulkan_on_windows { wgpu::Backends::VULKAN } else { wgpu::Backends::DX12 }
    } else if cfg!(target_os = "macos") {
        wgpu::Backends::METAL
    } else {
        wgpu::Backends::VULKAN
    }
}

impl Gpu {
    /// A device with no window, for captures and tests.
    pub fn headless(options: &GpuOptions) -> Result<Self, GpuError> {
        Self::create(options, None).map(|(gpu, _)| gpu)
    }

    /// A device and a surface for a window.
    ///
    /// # Safety
    /// The window behind `handles` must outlive the returned surface.
    pub unsafe fn for_window(
        options: &GpuOptions,
        handles: WindowHandles,
    ) -> Result<(Self, wgpu::Surface<'static>), GpuError> {
        let (gpu, surface) = Self::create(options, Some(handles))?;
        Ok((gpu, surface.expect("a surface was requested")))
    }

    fn create(
        options: &GpuOptions,
        handles: Option<WindowHandles>,
    ) -> Result<(Self, Option<wgpu::Surface<'static>>), GpuError> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = backends(options);
        let instance = wgpu::Instance::new(descriptor);

        let surface = match handles {
            None => None,
            Some(h) => {
                let target = wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: Some(h.display),
                    raw_window_handle: h.window,
                };
                // SAFETY: guaranteed by the caller of `for_window`.
                let surface = unsafe { instance.create_surface_unsafe(target) }
                    .map_err(|e| GpuError::Failed(format!("surface: {e}")))?;
                Some(surface)
            }
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: options.software,
            compatible_surface: surface.as_ref(),
            ..Default::default()
        }))
        .map_err(|e| GpuError::NoAdapter(e.to_string()))?;
        let info = adapter.get_info();

        let needed = required_limits();
        let offered = adapter.limits();
        let mut failures = Vec::new();
        needed.check_limits_with_fail_fn(&offered, false, |name, wanted, got| {
            failures.push(format!("{name}: needs {wanted}, has {got}"));
        });
        if !failures.is_empty() {
            return Err(GpuError::TooOld(format!("{} ({}): {}", info.name, info.backend, failures.join("; "))));
        }

        let timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let mut features = wgpu::Features::empty();
        if timestamps {
            features |= wgpu::Features::TIMESTAMP_QUERY;
        }

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("vellmark"),
            required_features: features,
            required_limits: needed,
            ..Default::default()
        }))
        .map_err(|e| GpuError::Failed(format!("{} ({}): {e}", info.name, info.backend)))?;

        Ok((Self { instance, adapter, device, queue, info, timestamps }, surface))
    }

    /// One line for logs and reports.
    pub fn describe(&self) -> String {
        format!("{} ({}, {:?})", self.info.name, self.info.backend, self.info.device_type)
    }

    /// True for a software renderer. Reports written on one are marked as not being
    /// performance evidence.
    pub fn is_software(&self) -> bool {
        self.info.device_type == wgpu::DeviceType::Cpu
    }
}
