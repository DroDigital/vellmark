//! The mixer runs on the audio thread, which must never allocate.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

#[test]
fn the_mixer_never_allocates() {
    let controls = vellmark_audio::Controls::new();
    for p in [&controls.wind, &controls.rain, &controls.water, &controls.trees, &controls.daylight] {
        p.set(1.0);
    }
    let mut mixer = vellmark_audio::Mixer::new(controls.clone(), 5);
    let mut buffer = vec![0.0f32; 960];
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for i in 0..2000 {
        controls.rain.set((i % 7) as f32 / 7.0);
        mixer.render(&mut buffer);
    }
    assert_eq!(ALLOCATIONS.load(Ordering::Relaxed), before, "the mixer allocated");
}
