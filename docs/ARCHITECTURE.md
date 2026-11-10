# Architecture

How the code is laid out, and how a frame is made. The rules it follows are in
`full-game/docs/brief/1a-core-constraints.md` and `docs/TECH.md`; this file says where
each of them lives.

## Crates

One Cargo workspace, in `crates/`. An arrow means "depends on".

| Crate | What it holds | Depends on |
|---|---|---|
| `sim` | The rules. Fixed-step `World`, `PlayerInput`, the immutable `Snapshot`. No SDL, no GPU | `glam` |
| `content` | Binary formats and their readers: packs, tiles, string tables, camera paths, the glyph atlas. Permanent IDs | `bytemuck` |
| `platform` | SDL3: window, events, controllers. The action layer and the simulated device | `sdl3` |
| `gpu` | One `wgpu` device, pass timers, the pipeline registry, texture readback | `wgpu` |
| `ui` | The interface: text and rectangles laid into a `Canvas`. Draws nothing | `content` |
| `render` | The renderer: draws a list of views and the canvas. Shader library | `gpu`, `sim`, `content`, `ui` |
| `world` | Terrain and streaming. Empty until Phase S02 | `content` |
| `audio` | The mixer. Empty until Phase S03 | |
| `storefront` | The Steam and itch trait. Empty until after the Showcase | |
| `bake` | The offline content tool: authored data and third-party files in, packs out | `content`, `ron`, `fdsm`, `ttf-parser` |
| `harness` | Proof tooling: captures, reference images, the shimmer check, source checks | most of the above |
| `app` | The executable: boot, the frame loop, the performance run, development tools | all |

`sim` knows nothing of drawing, and `render` reads only snapshots. That is what lets the
bots play the real game headless, and lets a failed test be replayed.

## A frame

```text
platform.poll ──► RawInput ──► Bindings ──► ActionState
                                               │  one PlayerInput for each step
FixedClock.advance(frame time) ──► n steps ──► World.step ──► Snapshot (immutable)
                                                                 │
                 previous Snapshot ──lerp(alpha)──► Camera for each view
                                                                 │
Renderer.render(views, canvas) ──► passes, each with a GPU timer ──► present
```

- The simulation runs at 60 steps a second whatever the frame rate. A frame runs zero or
  more whole steps and draws between the last two snapshots.
- Looking is applied once a frame; movement is applied to every step.
- The world holds a list of players and each has a view. The renderer draws a list of
  views into viewports of one target. The Showcase uses one.

## Formats

Authoring formats are RON and plain text under `assets/`. The game never reads them.
`bake` writes one pack, `base.vpk`, and the game reads only packs.

| Format | Magic | Defined in |
|---|---|---|
| Pack: header, assets in `(kind, id)` order, table of contents at the end | `VLMKPACK` | `content/src/pack.rs` |
| String table | `VLMKSTR1` | `content/src/strings.rs` |
| Camera path | `VLMKCAM1` | `content/src/camera_path.rs` |
| Glyph atlas: two faces, 95 glyphs each, distance fields with five levels | `VLMKGLY1` | `content/src/glyph_atlas.rs` |

- **IDs are permanent.** An asset's ID is the 64-bit FNV-1a hash of its authored key. A
  string's ID is the hash of its key, written `sid!("key")` in code. Keys are added and
  never renamed.
- **Tiles.** The world is addressed in 64 m tiles with a level and a layer
  (`content/src/tile.rs`). The Showcase loads its tiles whole; the full game streams the
  same tiles.
- **Baking is deterministic.** Inputs are read in sorted order, assets are written in ID
  order, and nothing in a pack records a file name or a time. Slow results are cached by
  the hash of their inputs, and a test checks that a cached bake equals a fresh one.

## Text

No player-facing text is written in code. The interface accepts only `Shown`, which is
made from the string table. Development tools use `dev_text`, which exists only with the
`dev` feature and only in modules named `dev`; a test reads the source tree and fails on
any other use, and on any `sid!` key that is missing from the tables.

The messages shown when the game cannot start are the exception in where they are
stored, not in where they are written: they are authored in `assets/text` and baked into
the executable, because they must be readable when the packs are not.

## Pipelines

Every pipeline is made through `gpu::PipelineRegistry` during startup or loading. The
registry is then sealed. A pipeline made after that is counted and logged, and the
capture tool and the performance report both fail on a count above zero.

## Development and shipping builds

The `dev` feature of `app` is on by default. It adds hot reload, the capture tool, the
console, the free camera, and the profiler view. `cargo build -p vellmark-app
--no-default-features` is the shipping build and contains none of them; CI builds it on
every push.

A development build run from its repository bakes on launch and reads shaders from disk.
The same build run anywhere else, as on the owner's Mac, uses the packs and shaders it
was packaged with.

## Threads

Today: the main thread does everything, and `rayon` is used by the bake. The audio
callback, the streaming thread, and job work arrive with the systems that need them.

## Determinism

`World::step` is deterministic on one machine: the same configuration and inputs give
the same state bit for bit, and a test runs a thousand steps twice to prove it. Across
machines it is not yet guaranteed, because `f32::sin` and `cos` are not specified to the
last bit. Saves store state and never depend on replaying inputs, so this matters only
for cross-machine replays, which nothing needs yet.
