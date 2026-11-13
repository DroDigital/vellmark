use std::collections::BTreeMap;
use std::path::PathBuf;

include!("shader_pre.rs");
include!(concat!(env!("OUT_DIR"), "/embedded_shaders.rs"));

/// Shader sources by name. Shipping builds use the sources embedded when the game was
/// compiled. Development builds may point the library at the `shaders` folder, and then
/// the file on disk wins, which is what makes hot reload possible.
pub struct ShaderLibrary {
    embedded: BTreeMap<&'static str, &'static str>,
    disk: Option<PathBuf>,
}

impl Default for ShaderLibrary {
    fn default() -> Self {
        Self::embedded()
    }
}

impl ShaderLibrary {
    pub fn embedded() -> Self {
        Self {
            embedded: EMBEDDED_SHADERS.iter().copied().collect(),
            disk: None,
        }
    }

    #[cfg(feature = "dev")]
    pub fn with_folder(folder: impl Into<PathBuf>) -> Self {
        Self {
            disk: Some(folder.into()),
            ..Self::embedded()
        }
    }

    pub fn folder(&self) -> Option<&std::path::Path> {
        self.disk.as_deref()
    }

    fn raw(&self, name: &str) -> Option<String> {
        if let Some(dir) = &self.disk {
            if let Ok(text) = std::fs::read_to_string(dir.join(format!("{name}.wgsl"))) {
                return Some(text);
            }
        }
        self.embedded.get(name).map(|s| (*s).to_owned())
    }

    /// The full text of a shader, with its imports expanded.
    pub fn source(&self, name: &str) -> Result<String, String> {
        expand_shader(name, &|n| self.raw(n))
    }

    /// Compiles a shader. An error in the source is returned, not raised, so that a bad
    /// edit during hot reload leaves the old pipeline in place.
    pub fn module(
        &self,
        gpu: &vellmark_gpu::Gpu,
        name: &str,
    ) -> Result<vellmark_gpu::wgpu::ShaderModule, String> {
        use vellmark_gpu::wgpu;
        let source = self.source(name)?;
        let scope = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
        match pollster_block(scope.pop()) {
            Some(error) => Err(format!("{name}.wgsl: {error}")),
            None => Ok(module),
        }
    }
}

fn pollster_block<F: std::future::Future>(future: F) -> F::Output {
    // Error scopes resolve at once on native backends; a tiny executor is enough.
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    fn noop(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    // SAFETY: the vtable functions do nothing and hold no state.
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return value;
        }
        std::thread::yield_now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_expand_once() {
        let lib = ShaderLibrary::embedded();
        let full = lib.source("opaque").unwrap();
        assert_eq!(full.matches("struct ViewUniform").count(), 1);
        assert!(!full.contains("#import"));
    }

    #[test]
    fn a_missing_shader_is_an_error() {
        assert!(ShaderLibrary::embedded().source("no_such_shader").is_err());
    }

    #[test]
    fn import_cycles_are_reported() {
        let source = |n: &str| match n {
            "a" => Some("#import b\n".to_owned()),
            "b" => Some("#import a\n".to_owned()),
            _ => None,
        };
        assert!(
            expand_shader("a", &source)
                .unwrap_err()
                .contains("imports itself")
        );
    }
}
