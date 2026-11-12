//! The windowed game: one window, the fixed-step simulation, and the frame loop.

use crate::args::Args;
use crate::boot::{self, Content};
use crate::log;
use crate::perf::{PerfRun, PerfStep};
use glam::Vec3;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vellmark_content::{sid, AssetId, AssetKind, CameraPath};
use vellmark_gpu::{wgpu, Gpu, GpuOptions, PassTimers, PipelineRegistry, WindowHandles};
use vellmark_platform::{
    Action, ActionState, Bindings, InputSource, Platform, PlatformEvent, RawInput, Window, WindowOptions,
};
use vellmark_render::{Camera, FrameInput, Renderer, ShaderLibrary, View};
use vellmark_ui::Canvas;
use vellmark_sim::{buttons, FixedClock, PlayerInput, Snapshot, World, WorldConfig};

/// The scene a new game starts in. The yard, until the valley exists.
const START_SCENE: &str = "yard";

struct Screen {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl Screen {
    fn new(gpu: &Gpu, surface: wgpu::Surface<'static>, size: (u32, u32), uncapped: bool) -> Result<Self, String> {
        let caps = surface.get_capabilities(&gpu.adapter);
        let mut config = surface
            .get_default_config(&gpu.adapter, size.0.max(1), size.1.max(1))
            .ok_or("the graphics card cannot draw to this window")?;
        config.format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
        config.present_mode = if uncapped {
            // The performance run measures real frame time, so it does not wait for the display.
            [wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox]
                .into_iter()
                .find(|m| caps.present_modes.contains(m))
                .unwrap_or(wgpu::PresentMode::Fifo)
        } else {
            wgpu::PresentMode::Fifo
        };
        config.desired_maximum_frame_latency = 2;
        surface.configure(&gpu.device, &config);
        Ok(Self { surface, config })
    }

    fn resize(&mut self, gpu: &Gpu, size: (u32, u32)) {
        self.config.width = size.0.max(1);
        self.config.height = size.1.max(1);
        self.surface.configure(&gpu.device, &self.config);
    }

    fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// The next frame to draw into, or `None` when this frame should be skipped.
    fn acquire(&mut self, gpu: &Gpu) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => Some(t),
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                drop(t);
                self.surface.configure(&gpu.device, &self.config);
                None
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&gpu.device, &self.config);
                None
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => None,
        }
    }
}

/// Turns a frame's actions into the input for one simulation step.
fn player_input(actions: &ActionState, bindings: &Bindings, dt: f32, first_step: bool) -> PlayerInput {
    let (move_x, move_y) = actions.movement();
    // Looking is applied once a frame, however many steps the frame runs.
    let (look_yaw, look_pitch) = if first_step { actions.look(bindings, dt) } else { (0.0, 0.0) };
    let mut held = 0;
    for (action, bit) in [
        (Action::Sprint, buttons::SPRINT),
        (Action::Jump, buttons::JUMP),
        (Action::Sneak, buttons::SNEAK),
        (Action::Interact, buttons::INTERACT),
        (Action::Read, buttons::READ),
        (Action::MainHand, buttons::ATTACK_MAIN),
        (Action::OffHand, buttons::ATTACK_OFF),
    ] {
        if actions.held(action) {
            held |= bit;
        }
    }
    PlayerInput { move_x, move_y, look_yaw, look_pitch, buttons: held }
}

// `strings` is reloaded here and read once the interface shows text (Phase S03).
#[allow(unused_assignments)]
pub fn run(args: &Args, launched: Instant) -> Result<(), ()> {
    let system_strings = boot::system_strings();
    let fail = |message, detail: &str| {
        boot::fatal(&system_strings, message, detail);
        Err(())
    };

    #[cfg_attr(not(feature = "dev"), allow(unused_mut))]
    let Content { base, mut strings, assets } = match boot::load_content() {
        Ok(c) => c,
        Err(detail) => return fail(sid!("sys.packs.missing"), &detail),
    };

    let mut platform = match Platform::new() {
        Ok(p) => p,
        Err(detail) => return fail(sid!("sys.gpu.failed"), &detail),
    };
    log::line("platform ready");
    let (width, height) = args.windowed_size.unwrap_or((1280, 720));
    let mut window: Window = match platform.create_window(&WindowOptions {
        title: &strings.text(sid!("sys.window_title")),
        width,
        height,
        fullscreen: args.fullscreen,
        hidden: false,
    }) {
        Ok(w) => w,
        Err(detail) => return fail(sid!("sys.gpu.failed"), &detail),
    };
    log::line("window created");
    if args.perf_run && args.windowed_size.is_none() && !args.fullscreen {
        // The targets are set at 1920 by 1080 pixels, whatever the display's density.
        window.set_size_in_pixels(1920, 1080);
    }
    let (raw_window, raw_display) = match window.handles() {
        Ok(h) => h,
        Err(detail) => return fail(sid!("sys.gpu.failed"), &detail),
    };

    let gpu_options = GpuOptions { software: args.software, ..Default::default() };
    // SAFETY: `window` is declared before the surface and outlives it; both live until
    // this function returns, and the surface is dropped first.
    let created = unsafe { Gpu::for_window(&gpu_options, WindowHandles { window: raw_window, display: raw_display }) };
    let (gpu, surface) = match created {
        Ok(pair) => pair,
        Err(error) => return fail(boot::gpu_error_string(&error), &error.to_string()),
    };
    log::line(&format!("graphics: {}", gpu.describe()));

    let mut screen = match Screen::new(&gpu, surface, window.size_in_pixels(), args.perf_run) {
        Ok(s) => s,
        Err(detail) => return fail(sid!("sys.gpu.failed"), &detail),
    };

    // Every pipeline is made here, before play begins.
    let mut registry = PipelineRegistry::new();
    #[cfg(feature = "dev")]
    let shaders = match crate::paths::repo_root() {
        Some(root) => ShaderLibrary::with_folder(root.join("shaders")),
        None => ShaderLibrary::embedded(),
    };
    #[cfg(not(feature = "dev"))]
    let shaders = ShaderLibrary::embedded();
    let mut renderer = match Renderer::new(&gpu, &mut registry, &shaders, screen.config.format, &assets.borrow()) {
        Ok(r) => r,
        Err(detail) => return fail(sid!("sys.gpu.failed"), &detail),
    };
    let scene_key = format!("scene.{}", args.scene.as_deref().unwrap_or(START_SCENE));
    let scene = match vellmark_render::load_scene(&gpu, &base, &scene_key) {
        Ok((scene, on_gpu)) => {
            renderer.set_scene(&gpu, on_gpu);
            scene
        }
        Err(detail) => return fail(sid!("sys.packs.missing"), &detail),
    };
    // The ground the scene stands on, if it stands on any.
    let terrain = if scene.terrain.0 != 0 {
        match vellmark_world::Terrain::from_pack(&base) {
            Ok(t) => Some(std::sync::Arc::new(t)),
            Err(detail) => return fail(sid!("sys.packs.missing"), &detail),
        }
    } else {
        None
    };
    if let Some(t) = &terrain {
        let tiles: Vec<&vellmark_content::TerrainTile> = t.tiles().collect();
        if let Err(detail) = renderer.set_terrain(&gpu, &tiles, &assets.materials) {
            return fail(sid!("sys.gpu.failed"), &detail);
        }
    }
    registry.seal();
    log::line(&format!("renderer ready, presenting {:?} as {:?}", screen.config.format, screen.config.present_mode));
    let mut timers = PassTimers::new(&gpu, cfg!(feature = "dev") || args.perf_run);

    #[cfg(feature = "dev")]
    let mut dev_tools = crate::dev::DevTools::new(args.profiler);

    let mut perf = if args.perf_run {
        let path = base
            .get(AssetKind::CameraPath, AssetId::from_key("path.perf_tour"))
            .and_then(|b| CameraPath::from_bytes(b).ok());
        match path {
            Some(path) => Some(PerfRun::new(path, args.out.clone(), launched.elapsed())),
            None => return fail(sid!("sys.packs.missing"), "the performance path is not in the pack"),
        }
    } else {
        None
    };

    let mut world = World::new(WorldConfig::default());
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    #[cfg_attr(not(feature = "dev"), allow(unused_variables))]
    let breakable_places: Vec<(u64, Vec3)> = breakables.iter().map(|b| (b.id, Vec3::from(b.to_world.translation))).collect();
    world.set_scene(solids, breakables);
    // The simulation holds the feet; the scene's spawn is given as an eye.
    let spawn = scene.spawn;
    world.add_player(Vec3::new(spawn[0], spawn[1] - vellmark_sim::EYE_HEIGHT, spawn[2]), spawn[3]);
    if let Some(t) = &terrain {
        world.set_ground(t.clone());
    }
    let mut previous: Arc<Snapshot> = world.snapshot();
    let mut current: Arc<Snapshot> = previous.clone();

    let bindings = Bindings::default();
    let mut actions = ActionState::default();
    let mut raw: Vec<RawInput> = Vec::new();
    let mut clock = FixedClock::new();
    let mut last = Instant::now();
    let mut frame_index: u64 = 0;
    let mut mouse_captured = false;
    let mut glyph_layer = true;
    #[cfg_attr(not(feature = "dev"), allow(unused_mut))]
    let mut world_hour_override: Option<f32> = None;
    let mut canvas = Canvas::new();
    let mut close_reading = 0.0f32;
    let mut lamps_lit = true;
    let mut lamps = 1.0f32;
    #[cfg_attr(not(feature = "dev"), allow(unused_mut))]
    let mut tuning = vellmark_render::GlyphTuning::default();
    let started = Instant::now();

    'frames: loop {
        let now = Instant::now();
        let frame_time = now.duration_since(last);
        last = now;

        actions.begin_frame();
        raw.clear();
        platform.poll(&mut raw);
        #[cfg(feature = "dev")]
        dev_tools.console_input(&mut raw);
        for input in &raw {
            actions.feed(&bindings, *input);
        }
        for event in platform.drain_events() {
            match event {
                PlatformEvent::Quit => break 'frames,
                PlatformEvent::Resized(w, h) => screen.resize(&gpu, (w, h)),
                PlatformEvent::FocusLost => {
                    mouse_captured = false;
                    platform.capture_mouse(&window, false);
                }
                PlatformEvent::FocusGained => {}
            }
        }
        if actions.pressed(Action::Pause) {
            // Until the pause menu exists, Pause frees the mouse, and again leaves.
            if mouse_captured {
                mouse_captured = false;
                platform.capture_mouse(&window, false);
            } else {
                break 'frames;
            }
        }
        if !mouse_captured && perf.is_none() && (actions.pressed(Action::MainHand) || frame_index == 0) {
            mouse_captured = true;
            platform.capture_mouse(&window, true);
        }
        if cfg!(feature = "dev") && actions.pressed(Action::DevGlyphLayer) {
            glyph_layer = !glyph_layer;
        }
        #[cfg(feature = "dev")]
        {
            if actions.pressed(Action::DevConsole) {
                let open = dev_tools.toggle_console();
                platform.text_entry(&window, open);
            }
            for command in dev_tools.take_commands() {
                use crate::dev::Command;
                match command {
                    Command::Hour(h) => world_hour_override = h,
                    Command::GlyphLayer(on) => glyph_layer = on,
                    Command::Lamps(on) => lamps_lit = on,
                    Command::Reveal(v) => tuning.reveal = v,
                    Command::Contrast(v) => tuning.contrast = v,
                    Command::Look(n) => tuning.look = n,
                    Command::Heat(name, h) => {
                        world.set_heat(AssetId::from_key(&format!("{scene_key}.{name}")).0, h);
                    }
                    Command::Break(name, strength) => {
                        let id = AssetId::from_key(&format!("{scene_key}.{name}"));
                        let blow = current.views.first().map_or(Vec3::ZERO, |v| v.eye);
                        match world.break_thing_with(id.0, blow, strength) {
                            Some(letters) => log::line(&format!("broke `{name}` into {letters} letters")),
                            None => log::line(&format!("nothing called `{name}` is left to break")),
                        }
                    }
                    Command::Quit => break 'frames,
                }
            }
        }

        #[cfg(feature = "dev")]
        {
            dev_tools.frame(&gpu, &mut registry, &mut renderer, &shaders, &actions);
            if let Some(seen) = dev_tools.take_data_change() {
                // Rebake and take the new text. The pack on disk is rewritten too.
                match boot::load_content() {
                    Ok(content) => {
                        strings = content.strings;
                        dev_tools.note_reload("text and data", seen);
                    }
                    Err(e) => log::line(&format!("data reload failed, keeping the old data:\n{e}")),
                }
            }
        }

        #[cfg(feature = "dev")]
        dev_tools.tune(&mut tuning, &actions);
        // Holding Read eases from the realistic frame into the text frame, and back.
        let wanted = if actions.held(Action::Read) { 1.0 } else { 0.0 };
        let ease = (frame_time.as_secs_f32() / 0.35).min(1.0);
        close_reading += (wanted - close_reading).clamp(-ease, ease);
        #[cfg(feature = "dev")]
        if actions.pressed(Action::DevLook) {
            tuning.look = (tuning.look + 1) % 5;
        }
        #[cfg(feature = "dev")]
        if actions.pressed(Action::DevBreak) {
            // The nearest thing that is still whole, struck from where the player stands.
            let eye = current.views.first().map_or(Vec3::ZERO, |v| v.eye);
            let nearest = breakable_places
                .iter()
                .filter(|(id, _)| !current.broken.contains(id))
                .min_by(|a, b| a.1.distance_squared(eye).total_cmp(&b.1.distance_squared(eye)));
            if let Some(&(id, _)) = nearest {
                if let Some(letters) = world.break_thing(id, eye) {
                    log::line(&format!("broke a thing into {letters} letters"));
                }
            }
        }
        if cfg!(feature = "dev") && actions.pressed(Action::DevLamps) {
            lamps_lit = !lamps_lit;
        }
        // A lamp takes a moment to catch and a moment to die.
        let lamp_ease = (frame_time.as_secs_f32() / 0.6).min(1.0);
        lamps += (if lamps_lit { 1.0 } else { 0.0 } - lamps).clamp(-lamp_ease, lamp_ease);

        // Simulation: whole fixed steps, then draw between the last two snapshots.
        let plan = clock.advance(frame_time.as_nanos() as u64);
        for step in 0..plan.steps {
            let input = player_input(&actions, &bindings, frame_time.as_secs_f32(), step == 0);
            world.step(&[input]);
            previous = current;
            current = world.snapshot();
        }

        let mut hour = world_hour_override.unwrap_or(current.time_of_day);
        let mut camera = Camera::from_view(&previous.lerp_view(&current, 0, plan.alpha));
        #[cfg(feature = "dev")]
        dev_tools.override_camera(&mut camera, &actions, &bindings, frame_time.as_secs_f32());
        canvas.clear();
        #[cfg(feature = "dev")]
        dev_tools.draw(&mut canvas, &assets.glyphs, screen.size(), window.pixel_density(), frame_time, &vellmark_gpu::PassTiming::grouped(timers.latest()), &registry, &tuning);

        if let Some(run) = perf.as_mut() {
            match run.frame(frame_time, &vellmark_gpu::PassTiming::grouped(timers.latest()), 1.0) {
                PerfStep::Camera { eye, yaw, pitch, hour: h } => {
                    camera = Camera::looking(eye.into(), yaw, pitch);
                    hour = h;
                }
                PerfStep::Finished => {
                    let path = run.write_report(
                        &gpu.describe(),
                        gpu.is_software(),
                        screen.size(),
                        &format!("{:?}", screen.config.present_mode),
                        registry.created_during_play(),
                    );
                    log::line(&format!("performance report written to {}", path.display()));
                    break 'frames;
                }
            }
        }

        if let Some(frame) = screen.acquire(&gpu) {
            let target = frame.texture.create_view(&Default::default());
            let views = [View::full(camera)];
            let input = FrameInput {
                views: &views,
                time_of_day: hour,
                seconds: started.elapsed().as_secs_f32(),
                glyph_layer,
                close_reading: close_reading * close_reading * (3.0 - 2.0 * close_reading),
                dive: 0.0,
                lamps,
                tuning,
                reveal_mask: false,
                letters: &current.letters,
                letters_step: current.letters_step,
                broken: &current.broken,
                heat: &current.heat,
                weather: current.weather,
                ui: Some(&canvas),
            };
            let mut encoder = gpu.device.create_command_encoder(&Default::default());
            timers.begin_frame();
            renderer.render(&gpu, &mut encoder, &target, screen.size(), &input, &mut timers);
            timers.end_frame(&gpu, &mut encoder);
            gpu.queue.submit([encoder.finish()]);
            timers.after_submit();
            gpu.queue.present(frame);
        } else {
            // Minimised or hidden: do not spin.
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = gpu.device.poll(wgpu::PollType::Poll);
        timers.collect();

        if frame_index == 0 {
            log::line("first frame drawn");
        }
        frame_index += 1;
        if args.frames.is_some_and(|limit| frame_index >= limit) {
            break 'frames;
        }
    }

    if registry.created_during_play() > 0 {
        log::line(&format!(
            "fault: {} pipeline(s) were created during play: {:?}",
            registry.created_during_play(),
            registry.names_created_during_play()
        ));
    }
    // The surface must go before the window it draws into.
    drop(screen);
    window.set_fullscreen(false);
    Ok(())
}
