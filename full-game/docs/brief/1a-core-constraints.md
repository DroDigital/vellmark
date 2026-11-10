# Section 1: core constraints and stack

## 1. Non-negotiable constraints

**No game engine.** No Unity, Unreal, Godot, Bevy, Fyrox, macroquad, three-d, or similar frameworks. No UI frameworks. No scene graph or renderer crates. You write the renderer, world streaming, animation runtime, AI, audio mixer, UI, and tools, using only the focused libraries approved in Section 1.1.

**Distribution.** The game ships as a natively compiled executable on Steam and itch.io. Steam and the itch app handle installation and updates, so do not build a separate launcher or self-updater. Launching the game goes straight to the title sequence, and all settings live in the in-game menu.

**The stack.**

| Layer | Choice | Why |
|---|---|---|
| Language | Rust (stable), one Cargo workspace | Strong compile-time checks across a very large codebase, and one build system on every target |
| OS layer | SDL3 through the `sdl3` crate, statically linked and built from source | One battle-tested library for windows, input, gamepads, rumble, and audio output |
| GPU | `wgpu`, pinned to one exact version, with `static-dxc` | One renderer over DX12, Metal, and Vulkan, with no shader compiler DLL to ship |
| Shaders | WGSL, validated at build time, hot-reloaded in dev builds | Fast iteration on the look |
| Storefront | `steamworks` crate, behind a small `Storefront` trait | Steam features stay out of the game logic and out of the itch build |
| Content | Offline `bake` tool producing pack files | Generated content is built ahead of time, and updates stay small |

SDL3 is a platform layer, not a game engine. Use it only for OS access. Do not use its renderer or GPU API.

**Permitted crates only:** `sdl3`, `wgpu`, `steamworks` (Steam build only), `glam`, `bytemuck`, `serde` + `ron`, `rayon`, `pollster`, `png` (tools and screenshots only), plus the libraries listed in Section 1.1. Commit `Cargo.lock`, and check for the current stable version of each dependency when starting. If you believe another crate is essential, it must be low-level and permissively licensed, and you must justify it in `docs/DECISIONS.md`.

**Workspace layout.** `app` (entry point and frame loop), `platform` (SDL3 wrapper and device layer), `storefront` (trait plus Steam and itch implementations), `gpu`, `render`, `world` (terrain and streaming), `sim` (game rules, with no dependency on SDL or the GPU), `ui`, `audio`, `content` (data formats and pack reader), `bake` (offline content tool), and `harness` (headless bots and tests).

**Runtime model.** The main thread handles OS events and frame submission. The simulation runs at a fixed timestep and hands each view an immutable snapshot to draw. `rayon` runs job work, a dedicated thread handles streaming I/O, and the SDL audio callback never allocates or locks. `sim` must run headless so bots can play the real game.

**No shader stutter.** Create every render pipeline during startup or loading, and use `wgpu::PipelineCache` where the backend supports it. A pipeline created during gameplay is a test failure.

**Platforms.**

- **Windows:** x86_64, Windows 10 or later, DX12 by default with Vulkan as a fallback setting.
- **macOS:** Apple Silicon only, macOS 13 or later, on Metal.
- **Linux and Steam Deck:** x86_64 native builds on Vulkan, working on both X11 and Wayland. Compile against the Steam Linux Runtime SDK, using the newest runtime Valve has made stable at release time.
- All three are release targets from Phase 00 onward. Use only GPU features and limits available on all three backends. No platform may lag behind or be left for the end.
- CI (GitHub Actions) compiles and runs fast tests on all three for every push. Full bakes, captures, bots, and packaging run nightly and before each gate, to limit CI cost. Hosted runners usually have no real GPU, so CI captures may use software rendering and never count as performance evidence.
- Show a readable error message, in place of a crash, if the GPU or driver cannot run the game.
- Saves, settings, and logs go in each platform's standard user data location. Save files must be byte-compatible across operating systems so Steam Cloud can move them between machines. Write saves atomically so a crash cannot corrupt one.
- Handle high-DPI displays, ultrawide, 16:10, borderless fullscreen, and case-sensitive file systems.
- A panic hook writes a local crash log and attempts an emergency save. Keep separate debug symbols for each release.
- Shipping profile: full LTO and one codegen unit. Dev profile: optimized dependencies and fast incremental builds of the game crates.

**Pack files (follow SteamPipe's guidance).** Steam splits files into roughly 1 MB chunks, and pack layout decides how large every patch is.

- Baking must be deterministic: identical inputs produce identical bytes.
- Each generated asset uses its own stable seed, so changing one asset never changes the others.
- Group assets by region or feature, with packs of 1 to 2 GB at most. New content goes in new packs.
- Keep asset order stable, use one table of contents at the start or end of the file, and embed no filenames or build timestamps.
- Do not compress or encrypt whole packs. If size requires compression, compress per asset.
- RON and plain text are authoring formats only. The bake writes a custom versioned binary format, and the game never parses RON at runtime.
- Every world object, NPC, quest, and item has a permanent ID that survives patches, so an update can never break an existing save.


**Approved third-party content only.** Meshes, textures, animations, fonts, sounds, shaders, and libraries may come only from the sources approved in Section 1.1, under its license rules. Everything else is generated by code or authored as text data in this repo.

**ASCII purity.** Every surface in the world is composed of the 95 printable ASCII characters (0x20 to 0x7E), at a scale too fine to see until the player is close, and the HUD, menus, and map are typeset in the same characters, with the accented Latin letters that `4c-content-rules.md` allows for translated text. This is the game's identity. Do not add runes, icons, or sprites, and never leave a surface without its fine print.

**No telemetry or online services.** The game collects no analytics and needs no account. The only online features are those Steam provides.

**Never fake completion.** No stubs presented as features, no placeholder content at a phase gate, no gate marked passed without its evidence on disk.
