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
use vellmark_render::{Camera, FrameInput, Governor, Preset, Renderer, ShaderLibrary, View};
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
        (Action::Sidestep, buttons::SIDESTEP),
        (Action::NextWord, buttons::NEXT_WORD),
        (Action::PreviousWord, buttons::PREVIOUS_WORD),
    ] {
        if actions.held(action) {
            held |= bit;
        }
    }
    PlayerInput { move_x, move_y, look_yaw, look_pitch, buttons: held, typed: 0, favourite: 0, say: 0 }
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
    // The player's settings, if they have saved any.
    let settings_path = crate::paths::user_data_dir().join("settings.ron");
    let mut settings = match std::fs::read_to_string(&settings_path) {
        Ok(text) => vellmark_platform::Settings::parse(&text).unwrap_or_else(|e| {
            log::line(&format!("the settings could not be read, so the defaults are used: {e}"));
            Default::default()
        }),
        Err(_) => Default::default(),
    };
    // The preset: as asked on the command line, else as set, else picked by the first
    // launch from the graphics hardware and kept.
    let mut preset = match args.preset.as_deref().and_then(Preset::parse).or_else(|| settings.preset.as_deref().and_then(Preset::parse)) {
        Some(p) => p,
        None => {
            let p = Preset::first_launch(gpu.is_software(), gpu.is_discrete());
            settings.preset = Some(p.name().into());
            if let Err(e) = crate::paths::write_atomically(&settings_path, settings.to_text().as_bytes()) {
                log::line(&format!("the settings could not be saved: {e}"));
            }
            log::line(&format!("first launch: the {p:?} preset, from {}", gpu.describe()));
            p
        }
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
    // Always on where the GPU can time its passes: the governor reads them.
    let mut timers = PassTimers::new(&gpu, true);

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
    world.set_trunks(vellmark_render::tree_trunks(&base, &scene));
    world.set_readables(vellmark_render::scene_readables(&base, &scene_key));
    let rules = vellmark_render::word_rules(&base);
    world.set_targets(vellmark_render::scene_targets(&base, &scene_key, &rules));
    world.set_word_rules(rules);
    // The simulation holds the feet; the scene's spawn is given as an eye.
    let spawn = scene.spawn;
    world.add_player(Vec3::new(spawn[0], spawn[1] - vellmark_sim::EYE_HEIGHT, spawn[2]), spawn[3]);
    if let Some(t) = &terrain {
        world.set_ground(t.clone());
    }
    for c in vellmark_render::scene_creatures(&scene) {
        world.add_creature(c);
    }
    // The loot table, and the plain iron sword a player starts with.
    if let Some(loot) = base.get(vellmark_content::AssetKind::Table, vellmark_content::AssetId::from_key(vellmark_content::LOOT_KEY)).and_then(|b| vellmark_content::LootTable::from_bytes(b).ok()) {
        let sword = vellmark_sim::items::Item::plain(&loot, "sword", "iron", 1);
        world.set_loot(loot);
        let me = world.snapshot().players[0].id;
        if let Some(index) = sword.and_then(|s| world.give_item(me, s)) {
            world.equip(me, index);
        }
    }
    // The Pages that write as they are watched, and how far each stands written on its
    // surface, so that a page is written again only when it changes.
    let page_writing = vellmark_render::scene_page_writing(&base, &scene_key);
    let mut page_shown: Vec<u32> = vec![u32::MAX; page_writing.len()];
    // Who says what.
    if let Some(table) = base.get(AssetKind::Table, AssetId::from_key(vellmark_content::DIALOGUE_KEY)).and_then(|b| vellmark_content::DialogueTable::from_bytes(b).ok()) {
        world.set_dialogue(table);
    }
    // The game is saved apart from the settings; `--continue` starts from it.
    let save_path = crate::paths::user_data_dir().join(format!("{}.vsave", scene_key.trim_start_matches("scene.")));
    if args.continue_game {
        match std::fs::read(&save_path).map_err(|e| e.to_string()).and_then(|b| vellmark_sim::save::SaveGame::from_bytes(&b)) {
            Ok(save) => {
                world.load(&save);
                log::line("the saved game is loaded");
            }
            Err(e) => log::line(&format!("no saved game to continue: {e}")),
        }
    }
    let mut previous: Arc<Snapshot> = world.snapshot();
    let mut current: Arc<Snapshot> = previous.clone();

    let mut bindings = settings.input.bindings();
    let mut actions = ActionState::default();
    let mut raw: Vec<RawInput> = Vec::new();
    let mut clock = FixedClock::new();
    let mut last = Instant::now();
    let mut frame_index: u64 = 0;
    let mut mouse_captured = false;
    let mut glyph_layer = true;
    #[cfg_attr(not(feature = "dev"), allow(unused_mut))]
    let mut world_hour_override: Option<f32> = None;
    // A gust a tool asked for: which sentence, and when it began, in seconds since start.
    #[allow(unused_mut)]
    let mut forced_gust: Option<(u32, f32)> = None;
    // The title sequence, where the scene has a camera path for it: the world holds still
    // and the camera follows the path until a button is pressed with the title up.
    let title_path = base.get(AssetKind::CameraPath, AssetId::from_key(&format!("path.title_{}", scene_key.trim_start_matches("scene.")))).and_then(|b| CameraPath::from_bytes(b).ok());
    let mut title = match &title_path {
        Some(path) if !args.no_title && perf.is_none() && args.frames.is_none() => {
            // The game begins at the hour the title leaves off, unless a save says otherwise.
            if !args.continue_game {
                if let Some(last) = path.keys.last() {
                    world.set_time_of_day(last.hour);
                    previous = world.snapshot();
                    current = previous.clone();
                }
            }
            Some(vellmark_ui::title::Title::new(&assets.glyphs, &strings, settings.reduced_motion))
        }
        _ => None,
    };
    let mut canvas = Canvas::new();
    let mut close_reading = 0.0f32;
    let mut lamps_lit = true;
    let mut lamps = 1.0f32;
    #[cfg_attr(not(feature = "dev"), allow(unused_mut))]
    let mut tuning = vellmark_render::GlyphTuning::default();
    let started = Instant::now();
    renderer.set_quality(preset.quality());
    // The one governor: it holds the frame time, and logs every step it takes.
    let mut governor = Governor::new(1000.0 / 60.0);
    let mut relief = vellmark_render::Relief::default();
    log::line(&format!("preset {preset:?}"));

    let mut sound = crate::sound::Sound::start(&platform, &scene);
    if let Some(table) = base.get(vellmark_content::AssetKind::Table, vellmark_content::AssetId::from_key(vellmark_content::WORDS_KEY)).and_then(|b| vellmark_content::WordTable::from_bytes(b).ok()) {
        sound.set_words(&table);
    }
    if let Some(table) = base.get(AssetKind::Table, AssetId::from_key(vellmark_content::VOICES_KEY)).and_then(|b| vellmark_content::VoiceTable::from_bytes(b).ok()) {
        sound.set_voices(table);
    }
    // The line last murmured, so that each is murmured once as it begins.
    let mut murmured: Option<(u64, u64)> = None;
    // Letters typed while a word charges, waiting for the next step.
    let mut typing = false;
    let mut typed = String::new();
    // Doors that words have opened swing on their hinges: each door's place as built, and
    // how far open it is now, 0 to 1.
    let door_places: Vec<(vellmark_content::AssetId, glam::Mat4)> = scene
        .objects
        .iter()
        .map(|o| {
            let t = o.transform;
            (o.key, glam::Mat4::from_cols_array(&[t[0], t[4], t[8], 0.0, t[1], t[5], t[9], 0.0, t[2], t[6], t[10], 0.0, t[3], t[7], t[11], 1.0]))
        })
        .collect();
    let mut door_open: std::collections::HashMap<u64, f32> = std::collections::HashMap::new();

    let mut view_model = vellmark_render::viewmodel::ViewModel::default();
    // Hit stop and the kick of a blow landing, in the first view.
    let mut hit_stop = 0.0f32;
    let mut held_view: Option<vellmark_sim::ViewSnap> = None;
    let mut kick = 0.0f32;
    // The pack, while it is open: its focus, and the repeat of a held direction. In solo
    // play the world holds still while it is open.
    let mut pack: Option<vellmark_ui::kit::Focus> = None;
    let mut pack_nav = vellmark_ui::kit::NavRepeat::default();
    let mut pack_nav_repeat = vellmark_ui::menu::MenuRepeat::default();
    // The favourites wheel while its button is held; the pairing chosen, for the next
    // step; and the pack's note after a pairing is kept or forgotten, with its time left.
    let mut wheel: Option<vellmark_ui::favourites_view::Wheel> = None;
    let mut favourite_pick: u8 = 0;
    let mut pack_note: Option<(vellmark_content::StringId, f32)> = None;
    // The pause menu and the settings, while open. The world holds still in solo play.
    let mut menu: Option<vellmark_ui::pages::Menu> = None;
    // A conversation: the choice the focus rests on, the choice taken this frame, the line
    // whose writing a press hurried to its end, and Interact held from that press, so that
    // it does not also go on to the next line.
    let mut talk_focus = vellmark_ui::kit::Focus::default();
    let mut talk_repeat = vellmark_ui::menu::MenuRepeat::default();
    let mut say: u8 = 0;
    let mut hurried: Option<u64> = None;
    let mut hold_interact = false;
    // Photo mode, while open, and how long its "Picture saved." has left to show.
    let mut photo: Option<vellmark_ui::photo::Photo> = None;
    let mut photo_note = 0.0f32;
    let mut take_picture = false;
    // The credits, while they roll: how far they have come.
    let credits_table = base.get(AssetKind::Table, AssetId::from_key(vellmark_content::CREDITS_KEY)).and_then(|b| vellmark_content::Credits::from_bytes(b).ok()).unwrap_or_default();
    let mut credits: Option<f32> = None;
    // Trade with a merchant, while it is open; and the last person spoken to, whose stock
    // a conversation's `trade` opens.
    let mut trade: Option<vellmark_ui::trade_view::Trade> = None;
    let mut last_talker: Option<u64> = None;
    // Captions for the sounds that matter, and whether a Redactor's approach is captioned.
    let mut captions = vellmark_ui::captions::Captions::default();
    let mut redactor_heard = false;
    let mut birds_hushed = false;
    // Reading by a press rather than a hold: whether Read is latched on.
    let mut read_latch = false;
    world.set_close_reading_slows(settings.close_reading_slows);
    vellmark_ui::kit::set_high_contrast(settings.high_contrast);
    vellmark_ui::set_letter_scale(settings.letter_size);
    vellmark_ui::kit::set_safe_colours(settings.safe_colours);
    renderer.set_fewer_flashes(settings.fewer_flashes);
    sound.set_volumes(settings.master_volume, settings.effects_volume, settings.ambience_volume, settings.music_volume);

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
        let menu_was_open = menu.is_some();
        let title_was_up = title.is_some();
        let mut menu_opened = false;
        // The title: any button hurries it, or once it is up, begins the game.
        if let Some(t) = title.as_mut() {
            if raw.iter().any(|r| matches!(r, RawInput::Key { down: true, .. } | RawInput::MouseButton { down: true, .. } | RawInput::PadButton { down: true, .. })) {
                t.press();
            }
            if t.update(frame_time.as_secs_f32()) == vellmark_ui::title::TitleState::Done {
                title = None;
                log::line("the title is over");
            }
        }
        // While a menu is open, it has the input.
        // The credits roll; up and down move them, back closes them.
        if let Some(scroll) = credits.as_mut() {
            let input = vellmark_ui::menu::MenuInput::from_keys(&raw);
            let d = window.pixel_density() * settings.interface_scale();
            let view = vellmark_ui::credits_view::CreditsView { atlas: &assets.glyphs, strings: &strings, credits: &credits_table, size: screen.size(), density: d };
            *scroll = (*scroll + (vellmark_ui::credits_view::ROLL * d) * frame_time.as_secs_f32() + input.step as f32 * 60.0 * d).max(0.0);
            if *scroll > view.height() + screen.size().1 as f32 {
                *scroll = 0.0;
            }
            if input.back || actions.pressed(Action::Pause) {
                credits = None;
            }
        }
        // A conversation's `trade` opens its speaker's stock.
        if let Some(t) = current.players.first().and_then(|p| p.talking.as_ref()) {
            last_talker = Some(t.talker);
        }
        if world.events().iter().any(|(_, e)| e == "trade") {
            if let Some(m) = last_talker.filter(|m| world.merchant(*m).is_some()) {
                trade = Some(vellmark_ui::trade_view::Trade { merchant: m, ..Default::default() });
            }
        }
        // Trading: up and down move along a side, left and right change sides, a press buys
        // or sells, and back leaves.
        if let (Some(t), Some(me)) = (trade.as_mut(), current.players.first().map(|p| p.id)) {
            let input = vellmark_ui::menu::MenuInput::read(&raw, &actions, &mut pack_nav_repeat, frame_time.as_secs_f32());
            let none = vellmark_content::LootTable::default();
            let view = vellmark_ui::trade_view::TradeView { atlas: &assets.glyphs, strings: &strings, loot: world.loot().unwrap_or(&none), size: screen.size(), density: window.pixel_density() * settings.interface_scale() };
            let rows = view.rows();
            use vellmark_ui::trade_view::Side;
            if input.change != 0 {
                t.side = if input.change < 0 { Side::Theirs } else { Side::Yours };
            }
            let theirs = world.merchant(t.merchant).map_or(0, |m| m.stock.len());
            let yours = world.inventory(me).map_or(0, |i| i.items.len());
            match t.side {
                Side::Theirs => t.theirs.step(input.step, theirs, rows),
                Side::Yours => t.yours.step(input.step, yours, rows),
            }
            if input.confirm {
                let done = match t.side {
                    Side::Theirs => world.buy(me, t.merchant, t.theirs.at),
                    Side::Yours => world.sell(me, t.merchant, t.yours.at),
                };
                t.refused = done.err();
                let (theirs, yours) = (world.merchant(t.merchant).map_or(0, |m| m.stock.len()), world.inventory(me).map_or(0, |i| i.items.len()));
                t.theirs.step(0, theirs, rows);
                t.yours.step(0, yours, rows);
            }
            if input.back || actions.pressed(Action::Inventory) {
                trade = None;
            }
        }
        if title.is_some() || photo.is_some() || credits.is_some() || trade.is_some() || wheel.is_some() {
        } else if menu.is_some() {
        } else if pack.is_some() && (actions.pressed(Action::Pause) || actions.pressed(Action::Inventory)) {
            pack = None;
            pack_note = None;
        } else if pack.is_some() && actions.pressed(Action::Favourites) {
            if let Some(kept) = current.players.first().and_then(|p| world.keep_hands(p.id)) {
                pack_note = Some((if kept { sid!("ui.favourites.kept") } else { sid!("ui.favourites.forgotten") }, 2.5));
            }
        } else if pack.is_none() && actions.pressed(Action::Favourites) && current.players.first().is_some_and(|p| p.reading == vellmark_sim::reading::ReadingSnap::None && p.talking.is_none()) {
            wheel = Some(Default::default());
        } else if actions.pressed(Action::Inventory) && current.players.first().is_some_and(|p| p.reading == vellmark_sim::reading::ReadingSnap::None && p.talking.is_none()) {
            pack = Some(Default::default());
            pack_nav = Default::default();
        } else if actions.pressed(Action::PhotoMode) && current.players.first().is_some_and(|p| p.talking.is_none()) {
            if let Some(p) = current.players.first() {
                photo = Some(vellmark_ui::photo::Photo::open(p.yaw, p.pitch));
            }
        } else if actions.pressed(Action::Pause) && menu.is_none() {
            menu = Some(vellmark_ui::pages::Menu::open());
            menu_opened = true;
        }
        // Photo mode: the panel by its fixed keys, the camera by the movement and look.
        if let (Some(ph), Some(view)) = (photo.as_mut(), current.views.first()) {
            let keys: Vec<RawInput> = raw.iter().filter(|r| !matches!(r, RawInput::Key { key: vellmark_platform::Key::Space | vellmark_platform::Key::E, .. })).copied().collect();
            if actions.pressed(Action::Inventory) {
                ph.panel = !ph.panel;
            }
            match ph.update(vellmark_ui::menu::MenuInput::from_keys(&keys)) {
                vellmark_ui::photo::PhotoOutcome::None => {}
                vellmark_ui::photo::PhotoOutcome::Save => take_picture = true,
                vellmark_ui::photo::PhotoOutcome::Leave => {
                    photo = None;
                    renderer.set_split(None);
                    renderer.set_exposure_scale(1.0);
                }
            }
            if let Some(ph) = photo.as_mut() {
                let (x, z) = actions.movement();
                let rise = actions.held(Action::Jump) as i32 as f32 - actions.held(Action::Sneak) as i32 as f32;
                let eye = view.eye;
                let ground = terrain.clone();
                ph.fly(Vec3::new(x, rise, z), actions.look(&bindings, frame_time.as_secs_f32()), frame_time.as_secs_f32(), &|at| {
                    ground.as_ref().and_then(|t| t.height_at(eye.x + at.x, eye.z + at.z)).map_or(f32::MIN, |g| g - eye.y)
                });
            }
        }
        let mut settings_changed = false;
        if let (Some(m), false) = (menu.as_mut(), menu_opened) {
            use vellmark_ui::pages::Outcome;
            let style = actions.pad.map_or(Default::default(), |p| platform.pad_style(p));
            let outcome = if m.capturing() {
                m.capture(&raw, &mut settings)
            } else {
                let input = vellmark_ui::menu::MenuInput::read(&raw, &actions, &mut m.repeat, frame_time.as_secs_f32());
                let len = m.entries(&settings, &bindings, actions.last_device, style).len();
                let view = vellmark_ui::menu::MenuView { atlas: &assets.glyphs, strings: &strings, size: screen.size(), density: window.pixel_density() * settings.interface_scale() };
                m.update(input, &mut settings, len, view.rows(len, &assets.glyphs))
            };
            match outcome {
                Outcome::None => {}
                Outcome::Resume => menu = None,
                Outcome::Changed => settings_changed = true,
                Outcome::Quit => break 'frames,
                Outcome::Credits => {
                    menu = None;
                    credits = Some(0.0);
                }
                Outcome::Photo => {
                    menu = None;
                    if let Some(p) = current.players.first() {
                        photo = Some(vellmark_ui::photo::Photo::open(p.yaw, p.pitch));
                    }
                }
                Outcome::Save => match crate::paths::write_atomically(&save_path, &world.save().to_bytes()) {
                    Ok(()) => {
                        log::line("the game is saved");
                        menu = None;
                    }
                    Err(e) => log::line(&format!("the game could not be saved: {e}")),
                },
                Outcome::Load => match std::fs::read(&save_path).map_err(|e| e.to_string()).and_then(|b| vellmark_sim::save::SaveGame::from_bytes(&b)) {
                    Ok(save) => {
                        world.load(&save);
                        previous = world.snapshot();
                        current = previous.clone();
                        log::line("the saved game is loaded");
                        menu = None;
                    }
                    Err(e) => log::line(&format!("no saved game: {e}")),
                },
            }
        }
        // The mouse is free while a menu is open, and taken again when it closes.
        if menu.is_some() && photo.is_none() && mouse_captured {
            mouse_captured = false;
            platform.capture_mouse(&window, false);
        }
        if (menu.is_none() || photo.is_some()) && title.is_none() && !mouse_captured && perf.is_none() && (actions.pressed(Action::MainHand) || frame_index == 0 || menu_was_open || title_was_up) {
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
            let settings_before = settings.clone();
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
                    Command::Rain(r) => world.set_rain(r),
                    Command::Fog(f) => world.set_fog(f),
                    Command::Preset(p) => {
                        preset = p;
                        renderer.set_quality(p.quality());
                        log::line(&format!("preset {p:?}"));
                    }
                    Command::Wind(n) => forced_gust = Some((n, started.elapsed().as_secs_f32())),
                    Command::Learn(word) => {
                        let id = current.players[0].id;
                        let all: Vec<String> = world.word_rules().words.iter().map(|w| w.spelling.clone()).collect();
                        let words = if word == "ALL" { all } else { vec![word] };
                        for w in words {
                            if world.learn(id, &w) {
                                log::line(&format!("learned {w}"));
                            } else {
                                log::line(&format!("no word {w}"));
                            }
                        }
                    }
                    Command::Save => match crate::paths::write_atomically(&save_path, &world.save().to_bytes()) {
                        Ok(()) => log::line("the game is saved"),
                        Err(e) => log::line(&format!("the game could not be saved: {e}")),
                    },
                    Command::Load => match std::fs::read(&save_path).map_err(|e| e.to_string()).and_then(|b| vellmark_sim::save::SaveGame::from_bytes(&b)) {
                        Ok(save) => {
                            world.load(&save);
                            previous = world.snapshot();
                            current = previous.clone();
                            log::line("the saved game is loaded");
                        }
                        Err(e) => log::line(&format!("no saved game: {e}")),
                    },
                    Command::Goto(x, z) => {
                        let id = current.players[0].id;
                        world.place_player(id, Vec3::new(x, 0.0, z), current.players[0].yaw);
                        log::line(&format!("went to {x} {z}"));
                    }
                    Command::Obelus(high) => {
                        let p = &current.players[0];
                        let ahead = Vec3::new(p.yaw.sin(), 0.0, -p.yaw.cos());
                        let side = Vec3::new(ahead.z, 0.0, -ahead.x);
                        let ground = |at: Vec3| terrain.as_ref().and_then(|t| t.height_at(at.x, at.z)).unwrap_or(p.feet.y);
                        let path: Vec<Vec3> = if high {
                            // Far off and high, crossing the view once.
                            let mid = p.feet + ahead * 900.0;
                            (0..=8).map(|k| mid + side * (k as f32 - 4.0) * 150.0 + Vec3::Y * 160.0).collect()
                        } else {
                            // Down the view from behind, low, a little to the side.
                            (0..=16).map(|k| p.feet + side * 2.5 + ahead * (k as f32 * 12.0 - 60.0)).map(|at| Vec3::new(at.x, ground(at) + 1.4, at.z)).collect()
                        };
                        world.add_redactor(vellmark_sim::redactor::Redactor { id: 1, path, speed: if high { 60.0 } else { 14.0 }, flown: 0.0, strike: (!high).then_some((2.4, 2.2)), caught: Vec::new() });
                        log::line(if high { "Obelus crosses the far sky" } else { "Obelus comes down the view" });
                    }
                    Command::Trade => {
                        const MERCHANT: u64 = 0x70b1;
                        if world.merchant(MERCHANT).is_none() {
                            world.add_merchant(MERCHANT);
                        }
                        trade = Some(vellmark_ui::trade_view::Trade { merchant: MERCHANT, ..Default::default() });
                    }
                    Command::Talk(speaker) => {
                        let p = &current.players[0];
                        let ahead = Vec3::new(p.yaw.sin(), 0.0, -p.yaw.cos()) * 2.0;
                        world.add_talker(vellmark_sim::dialogue::Talker { id: 0x7a1c, speaker: speaker.clone(), feet: p.feet + ahead, height: 1.75 });
                        log::line(&format!("{speaker} stands ahead"));
                    }
                    Command::Respawn => {
                        world.remove_creatures();
                        for c in vellmark_render::scene_creatures(&scene) {
                            world.add_creature(c);
                        }
                        log::line("the scene's creatures stand again");
                    }
                    Command::Sensitivity(n) => settings.input.mouse_sensitivity = n,
                    Command::InvertY(on) => settings.input.invert_y = on,
                    Command::Bind(action, button) => {
                        use vellmark_platform::prompts::{action_named, key_named, mouse_button_named, pad_button_named};
                        match action_named(&action) {
                            None => log::line(&format!("no action called `{action}`")),
                            Some(a) => {
                                if let Some(k) = key_named(&button) {
                                    settings.input.remap_key(a, k);
                                } else if let Some(m) = mouse_button_named(&button) {
                                    settings.input.remap_mouse(a, m);
                                } else if let Some(p) = pad_button_named(&button) {
                                    settings.input.remap_pad(a, p);
                                } else {
                                    log::line(&format!("no button called `{button}`"));
                                }
                            }
                        }
                    }
                    Command::Quit => break 'frames,
                }
            }
            if settings != settings_before {
                settings_changed = true;
            }
        }
        if settings_changed {
            bindings = settings.input.bindings();
            if let Some(p) = settings.preset.as_deref().and_then(Preset::parse).filter(|p| *p != preset) {
                preset = p;
                renderer.set_quality(p.quality());
                log::line(&format!("preset {p:?}"));
            }
            renderer.set_fewer_flashes(settings.fewer_flashes);
            sound.set_volumes(settings.master_volume, settings.effects_volume, settings.ambience_volume, settings.music_volume);
            world.set_close_reading_slows(settings.close_reading_slows);
            vellmark_ui::kit::set_high_contrast(settings.high_contrast);
            vellmark_ui::set_letter_scale(settings.letter_size);
            vellmark_ui::kit::set_safe_colours(settings.safe_colours);
            match crate::paths::write_atomically(&settings_path, settings.to_text().as_bytes()) {
                Ok(()) => log::line("settings saved"),
                Err(e) => log::line(&format!("the settings could not be saved: {e}")),
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
        // Close reading is the rules' to decide: Read with no text in focus.
        let wanted = if current.players.first().is_some_and(|p| p.reading == vellmark_sim::reading::ReadingSnap::Close) { 1.0 } else { 0.0 };
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

        // While a word charges, letters typed are read by the rules: the word for a
        // bonus, or its typo on purpose.
        let charging = current.players.first().is_some_and(|p| p.charge.iter().any(|&c| c > 0.0));
        #[cfg(feature = "dev")]
        let charging = charging && !dev_tools.console_open();
        if charging != typing {
            typing = charging;
            platform.text_entry(&window, typing);
            typed.clear();
        }
        if typing {
            typed.extend(raw.iter().filter_map(|r| match r {
                RawInput::Text(c) if c.is_ascii_alphabetic() => Some(*c),
                _ => None,
            }));
        }

        // The murmur: each line as it begins, in its speaker's voice; hushed when done.
        match current.players.first().and_then(|p| p.talking.as_ref()) {
            Some(t) => {
                if let Some(line) = t.line {
                    let key = (line, t.since.to_bits());
                    if murmured != Some(key) && t.choices.is_empty() {
                        murmured = Some(key);
                        sound.murmur(&t.by, &strings.text(vellmark_content::StringId(line)));
                    }
                }
            }
            None if murmured.is_some() => {
                murmured = None;
                sound.hush();
            }
            None => {}
        }
        // A conversation: choices are taken with the menu keys; a press on a line still
        // being written writes it out at once.
        if let (Some(t), None) = (current.players.first().and_then(|p| p.talking.as_ref()), &menu) {
            if t.choices.is_empty() {
                talk_focus = Default::default();
                let done = vellmark_ui::dialogue_view::written(&strings, t, current.seconds) || hurried == Some(t.since.to_bits());
                if actions.pressed(Action::Interact) && !done {
                    hurried = Some(t.since.to_bits());
                    hold_interact = true;
                }
            } else {
                let input = vellmark_ui::menu::MenuInput::read(&raw, &actions, &mut talk_repeat, frame_time.as_secs_f32());
                let rows = vellmark_ui::dialogue_view::DialogueView { atlas: &assets.glyphs, strings: &strings, size: screen.size(), density: window.pixel_density() * settings.interface_scale() }.rows();
                talk_focus.step(input.step, t.choices.len(), rows);
                if input.confirm {
                    say = talk_focus.at as u8 + 1;
                }
            }
        }
        if !actions.held(Action::Interact) {
            hold_interact = false;
        }

        // The favourites wheel: aimed by the mouse or either stick, and let go to take the
        // pairing aimed at; Escape lets it go without one.
        if let (Some(wh), Some(me)) = (wheel.as_mut(), current.players.first().map(|p| p.id)) {
            let (dx, dy) = actions.mouse_motion();
            wh.mouse(dx, dy);
            let (left, right) = actions.sticks();
            wh.stick(left.0, left.1);
            wh.stick(right.0, right.1);
            if actions.pressed(Action::Pause) {
                wheel = None;
            } else if !actions.held(Action::Favourites) {
                let count = world.inventory(me).map_or(0, |i| i.favourites.len().min(vellmark_ui::favourites_view::SLOTS));
                if let Some(k) = wh.slot(count) {
                    favourite_pick = k as u8 + 1;
                }
                wheel = None;
            }
        }
        if let Some((_, left)) = pack_note.as_mut() {
            *left -= frame_time.as_secs_f32();
            if *left <= 0.0 {
                pack_note = None;
            }
        }

        // The pack: move the focus, and equip what it rests on.
        if let (Some(focus), Some(me)) = (pack.as_mut(), current.players.first().map(|p| p.id)) {
            let len = world.inventory(me).map_or(0, |i| i.items.len());
            let rows = vellmark_ui::inventory_view::rows_shown(&assets.glyphs, screen.size(), window.pixel_density() * settings.interface_scale());
            focus.step(pack_nav.step(actions.movement().1, frame_time.as_secs_f32()), len, rows);
            if actions.pressed(Action::Interact) && focus.at < len {
                world.equip(me, focus.at);
            }
        }

        if settings.read_toggle && menu.is_none() && photo.is_none() && actions.pressed(Action::Read) {
            read_latch = !read_latch;
        }
        if !settings.read_toggle {
            read_latch = false;
        }
        // Simulation: whole fixed steps, then draw between the last two snapshots. In solo
        // play the world holds still while the pack or a menu is open.
        let plan = clock.advance(if pack.is_some() || menu.is_some() || title.is_some() || photo.is_some() || credits.is_some() || trade.is_some() || wheel.is_some() { 0 } else { frame_time.as_nanos() as u64 });
        for step in 0..plan.steps {
            let mut input = player_input(&actions, &bindings, frame_time.as_secs_f32(), step == 0);
            if step == 0 {
                input.say = std::mem::take(&mut say);
                input.favourite = std::mem::take(&mut favourite_pick);
            }
            if settings.read_toggle {
                input.buttons = (input.buttons & !vellmark_sim::buttons::READ) | if read_latch { vellmark_sim::buttons::READ } else { 0 };
            }
            if hold_interact {
                input.buttons &= !vellmark_sim::buttons::INTERACT;
            }
            let take: String = typed.drain(..typed.len().min(4)).collect();
            input.typed = PlayerInput::pack_typed(&take);
            world.step(&[input]);
            previous = current;
            current = world.snapshot();
            sound.heard(&current);
            sound.set_danger(&current);
            // Captions: each word written by its motif, and the sounds not to miss.
            if settings.captions {
                use vellmark_sim::Heard;
                let word = |w: &str| vellmark_content::StringId::from_key(&format!("caption.word.{}", w.to_ascii_lowercase()));
                for l in &current.landings {
                    let key = if l.typo { sid!("caption.sound.motif_wrong") } else { sid!("caption.sound.motif") };
                    captions.heard(key, Some(word(&l.word)));
                }
                for h in &current.heard {
                    match h {
                        Heard::Learned { word: w, .. } => {
                            if let Some(rule) = world.word_rules().words.get(*w) {
                                captions.heard(sid!("caption.sound.learned"), Some(word(&rule.spelling)));
                            }
                        }
                        Heard::Gather { .. } => captions.heard(sid!("caption.sound.gather"), None),
                        Heard::Call { .. } => captions.heard(sid!("caption.sound.call"), None),
                        Heard::Break { .. } => captions.heard(sid!("caption.sound.break"), None),
                        Heard::Mend { .. } => captions.heard(sid!("caption.sound.mend"), None),
                        _ => {}
                    }
                }
                let near = current.players.first().is_some_and(|p| current.redactors.iter().any(|r| r.at.distance(p.feet) < 200.0));
                if near && !redactor_heard {
                    captions.heard(sid!("caption.sound.redactor"), None);
                }
                redactor_heard = near;
                // By day the birds stop first, as the sound does.
                let hush = (6.0..20.0).contains(&current.time_of_day) && current.players.first().is_some_and(|p| current.redactors.iter().any(|r| r.at.distance(p.feet) < 240.0));
                if hush && !birds_hushed {
                    captions.heard(sid!("caption.sound.birds_stop"), None);
                }
                birds_hushed = hush;
            }
            // Hit stop: a blow that lands holds the striker's view a moment and kicks it.
            // Only the view: the shared world never pauses. Reduced screen response and
            // reduced motion leave the view alone.
            let response = !(settings.reduced_screen_response || settings.reduced_motion);
            if let (Some(me), true) = (current.players.first(), response) {
                for hit in current.hits.iter().filter(|h| h.player == me.id.0) {
                    if hit.hit_stop > hit_stop {
                        hit_stop = hit.hit_stop;
                        held_view = Some(previous.lerp_view(&current, 0, plan.alpha));
                    }
                    kick = kick.max(if hit.hit_stop > vellmark_sim::melee::HIT_STOP { 1.0 } else { 0.55 });
                }
            }
        }

        for (k, (id, pages)) in page_writing.iter().enumerate() {
            if let Some(&(_, n)) = current.pages_written.iter().find(|(r, _)| r == id) {
                if page_shown[k] != n {
                    renderer.set_written(&gpu, pages, n);
                    page_shown[k] = n;
                }
            }
        }
        let mut hour = world_hour_override.unwrap_or(current.time_of_day);
        let mut camera = Camera::from_view(&match held_view {
            Some(view) if hit_stop > 0.0 => view,
            _ => previous.lerp_view(&current, 0, plan.alpha),
        });
        hit_stop = (hit_stop - frame_time.as_secs_f32()).max(0.0);
        if hit_stop == 0.0 {
            held_view = None;
        }
        // The kick: a small jolt of the view that dies away in a tenth of a second.
        if kick > 0.0 {
            let t = started.elapsed().as_secs_f32();
            let jolt = kick * 0.006;
            camera.rotation = camera.rotation * glam::Quat::from_euler(glam::EulerRot::YXZ, (t * 91.0).sin() * jolt, (t * 73.0).sin() * jolt - jolt, 0.0);
            kick = (kick - frame_time.as_secs_f32() * 10.0).max(0.0);
        }
        // Redactors: the body where each flies, hidden while none does; what they struck
        // stands as its rules.
        let obelus = AssetId::from_key(&format!("{scene_key}.obelus"));
        match current.redactors.first() {
            Some(r) => renderer.scene_mut().place_kit(obelus, Some(vellmark_render::redactor_transform(r, started.elapsed().as_secs_f32()))),
            None => renderer.scene_mut().place_kit(obelus, None),
        }
        renderer.set_struck(&current.struck);
        // In photo mode, its own camera, near the player's eye; its split, exposure, and lens.
        if let (Some(ph), Some(view)) = (&photo, current.views.first()) {
            camera = Camera { eye: view.eye + ph.offset, rotation: ph.rotation(), fov_y: ph.fov.to_radians(), near: Camera::DEFAULT_NEAR };
            renderer.set_split(ph.split(screen.size()));
            renderer.set_exposure_scale(2f32.powf(ph.stops));
        }
        // Under the title, the camera follows its path, at the path's hour.
        if let (Some(t), Some(path)) = (&title, &title_path) {
            if let Some(key) = path.at(t.seconds()) {
                camera = Camera::looking(key.eye.into(), key.yaw, key.pitch);
                hour = key.hour;
            }
        }
        view_model.still = settings.reduced_motion;
        // The hands and what they hold, placed for this frame from the first player's view;
        // none under the title.
        if let Some(player) = current.players.first() {
            for (key, place) in view_model.place(player, camera.eye, camera.rotation, &current.arrows, started.elapsed().as_secs_f32(), frame_time.as_secs_f32()) {
                let hidden = title.is_some() || photo.as_ref().is_some_and(|p| !p.hands);
                renderer.scene_mut().place_kit(key, if hidden { None } else { place });
            }
        }
        #[cfg(feature = "dev")]
        dev_tools.override_camera(&mut camera, &actions, &bindings, frame_time.as_secs_f32());
        canvas.clear();
        if let (Some(t), Some(player), Some(loot)) = (&trade, current.players.first(), world.loot()) {
            if let (Some(m), Some(inv)) = (world.merchant(t.merchant), world.inventory(player.id)) {
                let style = actions.pad.map_or(Default::default(), |p| platform.pad_style(p));
                let (confirm, back) = vellmark_ui::pages::Menu::hint_keys(actions.last_device, style);
                vellmark_ui::trade_view::TradeView { atlas: &assets.glyphs, strings: &strings, loot, size: screen.size(), density: window.pixel_density() * settings.interface_scale() }
                    .draw(&mut canvas, t, m, inv, (Some(&confirm), Some(&back)));
            }
        } else if let Some(scroll) = credits {
            let d = window.pixel_density() * settings.interface_scale();
            vellmark_ui::credits_view::CreditsView { atlas: &assets.glyphs, strings: &strings, credits: &credits_table, size: screen.size(), density: d }.draw(&mut canvas, scroll);
        } else if let Some(ph) = &photo {
            let density = window.pixel_density() * settings.interface_scale();
            let style = actions.pad.map_or(Default::default(), |p| platform.pad_style(p));
            let hint = vellmark_platform::prompts::prompt_key(Action::Inventory, actions.last_device, style, &bindings)
                .map(|k| vellmark_ui::Shown::filled(&strings, sid!("ui.photo.hint"), &[("hide", vellmark_ui::Part::Shown(vellmark_ui::Shown::from_table(&strings, vellmark_content::StringId::from_key(&k))))]));
            let note = if photo_note > 0.0 { Some(vellmark_ui::Shown::from_table(&strings, sid!("ui.photo.saved"))) } else { hint };
            ph.draw(&mut canvas, &assets.glyphs, &strings, screen.size(), density, note);
            photo_note = (photo_note - frame_time.as_secs_f32()).max(0.0);
        } else if let Some(t) = &title {
            t.draw(&mut canvas, &assets.glyphs, &strings, screen.size(), window.pixel_density() * settings.interface_scale());
        } else if let Some(player) = current.players.first() {
            let style = actions.pad.map_or(Default::default(), |p| platform.pad_style(p));
            vellmark_ui::reading_view::ReadingView { atlas: &assets.glyphs, strings: &strings, size: screen.size(), density: window.pixel_density() * settings.interface_scale() }
                .draw(&mut canvas, player, &actions, &bindings, style);
            let ui_density = window.pixel_density() * settings.interface_scale();
            let talk_view = vellmark_ui::dialogue_view::DialogueView { atlas: &assets.glyphs, strings: &strings, size: screen.size(), density: ui_density };
            if let Some(t) = &player.talking {
                let now = if hurried == Some(t.since.to_bits()) { f64::MAX / 4.0 } else { current.seconds };
                let key = match (actions.last_device, t.choices.is_empty()) {
                    (vellmark_platform::DeviceKind::Pad, false) => Some(format!("prompt.pad.{}.south", vellmark_platform::prompts::style_name(style))),
                    _ => vellmark_platform::prompts::prompt_key(Action::Interact, actions.last_device, style, &bindings),
                };
                talk_view.draw(&mut canvas, t, now, talk_focus, key.as_deref());
            } else if let (Some(speaker), true) = (&player.talker_in_gaze, player.reading == vellmark_sim::reading::ReadingSnap::None && pack.is_none()) {
                if let Some(key) = vellmark_platform::prompts::prompt_key(Action::Interact, actions.last_device, style, &bindings) {
                    talk_view.draw_offer(&mut canvas, speaker, &key);
                }
            }
            if player.reading == vellmark_sim::reading::ReadingSnap::None && pack.is_none() && player.talking.is_none() {
                vellmark_ui::hud::Hud { atlas: &assets.glyphs, strings: &strings, rules: world.word_rules(), loot: world.loot(), size: screen.size(), density: window.pixel_density() * settings.interface_scale(), states: settings.creature_states }.draw(&mut canvas, player);
            }
            // Captions over play, below the menus.
            captions.update(frame_time.as_secs_f32());
            if settings.captions && menu.is_none() && pack.is_none() {
                captions.draw(&mut canvas, &assets.glyphs, &strings, screen.size(), window.pixel_density() * settings.interface_scale());
            }
            if let (Some(focus), Some(inv), Some(loot)) = (pack, world.inventory(player.id), world.loot()) {
                let key = |action| vellmark_platform::prompts::prompt_key(action, actions.last_device, style, &bindings);
                let (equip, close, keep) = (key(Action::Interact), key(Action::Inventory), key(Action::Favourites));
                vellmark_ui::inventory_view::InventoryView { atlas: &assets.glyphs, strings: &strings, loot, size: screen.size(), density: window.pixel_density() * settings.interface_scale(), note: pack_note.map(|n| n.0) }
                    .draw(&mut canvas, inv, focus, (equip.as_deref(), close.as_deref(), keep.as_deref()));
            }
            if let (Some(wh), Some(inv), Some(loot)) = (wheel, world.inventory(player.id), world.loot()) {
                let keep = vellmark_platform::prompts::prompt_key(Action::Favourites, actions.last_device, style, &bindings);
                vellmark_ui::favourites_view::FavouritesView { atlas: &assets.glyphs, strings: &strings, rules: world.word_rules(), loot, size: screen.size(), density: window.pixel_density() * settings.interface_scale() }
                    .draw(&mut canvas, &wh, inv, keep.as_deref());
            }
            if let Some(m) = &menu {
                let entries = m.entries(&settings, &bindings, actions.last_device, style);
                let (confirm, back) = vellmark_ui::pages::Menu::hint_keys(actions.last_device, style);
                let key = |k: &str| vellmark_ui::Part::Shown(vellmark_ui::Shown::from_table(&strings, vellmark_content::StringId::from_key(k)));
                let hint = vellmark_ui::Shown::filled(&strings, sid!("ui.menu.hint"), &[("confirm", key(&confirm)), ("back", key(&back))]);
                vellmark_ui::menu::MenuView { atlas: &assets.glyphs, strings: &strings, size: screen.size(), density: window.pixel_density() * settings.interface_scale() }
                    .draw(&mut canvas, m.title(), &entries, m.focus(), Some(hint));
            }
        }
        #[cfg(feature = "dev")]
        dev_tools.draw(&mut canvas, &assets.glyphs, screen.size(), window.pixel_density(), frame_time, &vellmark_gpu::PassTiming::grouped(timers.latest()), &registry, &tuning);

        if let Some(run) = perf.as_mut() {
            match run.frame(frame_time, &vellmark_gpu::PassTiming::grouped(timers.latest()), relief.scene_scale()) {
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
                        &format!("{preset:?}"),
                        &governor.log,
                    );
                    log::line(&format!("performance report written to {}", path.display()));
                    break 'frames;
                }
            }
        }

        sound.update(camera.eye, &current.weather, hour, terrain.as_deref(), &current.redactors);
        // Open doors swing, eased; closed ones swing back.
        for t in current.targets.iter() {
            let Some((key, place)) = door_places.iter().find(|(k, _)| k.0 == t.id) else { continue };
            let want = if t.effects.iter().any(|e| e == "open") { 1.0 } else { 0.0 };
            let now = door_open.entry(t.id).or_insert(0.0);
            if *now != want {
                *now += (want - *now).clamp(-frame_time.as_secs_f32() * 1.5, frame_time.as_secs_f32() * 1.5);
                let eased = *now * *now * (3.0 - 2.0 * *now);
                renderer.scene_mut().set_transform(*key, *place * glam::Mat4::from_rotation_y(-eased * 95f32.to_radians()));
            }
        }

        // Loose things stand where the rules put them, between the last two steps.
        if !std::sync::Arc::ptr_eq(&previous.props, &current.props) || plan.alpha > 0.0 {
            for (id, now) in current.props.iter() {
                let before = previous
                    .props
                    .iter()
                    .find(|(p, _)| p == id)
                    .map_or(*now, |(_, p)| *p);
                let (_, r0, t0) = before.to_scale_rotation_translation();
                let (_, r1, t1) = now.to_scale_rotation_translation();
                let place = glam::Mat4::from_rotation_translation(
                    r0.slerp(r1, plan.alpha),
                    t0.lerp(t1, plan.alpha),
                );
                renderer
                    .scene_mut()
                    .set_transform(vellmark_content::AssetId(*id), place);
            }
        }
        renderer.observe(&current, started.elapsed().as_secs_f32());
        if let Some(frame) = screen.acquire(&gpu) {
            let target = frame.texture.create_view(&Default::default());
            let views = [View::full(camera)];
            let input = FrameInput {
                views: &views,
                time_of_day: hour,
                seconds: started.elapsed().as_secs_f32(),
                glyph_layer,
                close_reading: if photo.as_ref().is_some_and(|p| p.view == vellmark_ui::photo::View::Text) { 1.0 } else { close_reading * close_reading * (3.0 - 2.0 * close_reading) },
                dive: 0.0,
                lamps,
                tuning,
                reveal_mask: false,
                letters: &current.letters,
                letters_step: current.letters_step,
                broken: &current.broken,
                heat: &current.heat,
                weather: current.weather,
                wind_gust: forced_gust.and_then(|(n, at)| {
                    let progress = (started.elapsed().as_secs_f32() - at) / 3.2;
                    (progress < 1.0).then_some((n, progress))
                }),
                relief,
                ui: Some(&canvas),
            };
            let mut encoder = gpu.device.create_command_encoder(&Default::default());
            timers.begin_frame();
            renderer.render(&gpu, &mut encoder, &target, screen.size(), &input, &mut timers);
            // The picture: the same frame without the panel, saved before this one shows.
            if std::mem::take(&mut take_picture) {
                let folder = settings.picture_folder.as_ref().map(std::path::PathBuf::from).unwrap_or_else(crate::picture::default_folder);
                match crate::picture::save(&gpu, &mut renderer, &input, screen.size(), &folder) {
                    Ok(path) => {
                        log::line(&format!("picture saved: {}", path.display()));
                        photo_note = 2.5;
                    }
                    Err(e) => log::line(&format!("the picture could not be saved: {e}")),
                }
            }
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
        // The frame's GPU time where the timers can say, its wall time where they cannot.
        let gpu_ms: f32 = timers.latest().iter().map(|p| p.milliseconds).sum();
        let frame_ms = if timers.active() && gpu_ms > 0.0 { gpu_ms } else { frame_time.as_secs_f32() * 1000.0 };
        let logged = governor.log.len();
        relief = governor.frame(frame_ms, frame_time.as_secs_f32(), started.elapsed().as_secs_f32(), close_reading > 0.0);
        for line in &governor.log[logged..] {
            log::line(&format!("governor: {line}"));
        }

        if frame_index == 0 {
            log::line("first frame drawn");
        }
        frame_index += 1;
        if args.frames.is_some_and(|limit| frame_index >= limit) {
            break 'frames;
        }
    }

    // The game is saved on the way out.
    if !args.perf_run && args.capture.is_none() {
        match crate::paths::write_atomically(&save_path, &world.save().to_bytes()) {
            Ok(()) => log::line("the game is saved"),
            Err(e) => log::line(&format!("the game could not be saved: {e}")),
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
