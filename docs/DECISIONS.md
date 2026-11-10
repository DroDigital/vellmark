# Decisions

Newest first. One entry for each choice that is not already made by the brief.

## 2026-11-10: Phase S00

**One branch for the session, not one for each task.** `CLAUDE.md` asks for a branch and
a pull request for each task. This cloud session was started on one assigned branch and
may push only there, so Phase S00 is one pull request with a commit for each task or
group of tasks. The task loop is otherwise followed. With several sessions in lanes, each
gets its own assigned branch, which gives the same isolation.

**Dependency versions.** Pinned exactly in the workspace manifest, checked against
crates.io on this day: `sdl3` 0.22.1, `wgpu` 30.0.1, `glam` 0.34.3, `bytemuck` 1.25.2,
`serde` 1.0.229, `ron` 0.12.3, `rayon` 1.12.0, `pollster` 1.0.1, `png` 0.18.1, `notify`
8.2.2 (the latest stable; 9 is a release candidate), `ttf-parser` 0.25.1, `fdsm` 0.8.0.

**Crates beyond the brief's list, and why.**

| Crate | Where | Why it is acceptable |
|---|---|---|
| `naga` 30.0.1 | Build script of `render` | Validates WGSL at build time, as the brief requires. It is `wgpu`'s own shader compiler and is already in the tree at this version. MIT or Apache-2.0 |
| `raw-window-handle` 0.6 | `platform` | Only to name the handle types passed from SDL to `wgpu`. Already in the tree through both. MIT or Apache-2.0 |
| `image`, `nalgebra` | `bake` | `fdsm`'s own image and point types. `image` is on the approved list for the bake; `nalgebra` (Apache-2.0) comes with `fdsm`. Bake only |

`naga_oil` is not used. Shader composition is one rule, `#import name`, in forty lines
shared by the build script and the renderer.

**SHA-256 is written in the bake.** The manifest needs it and no approved crate offers
it. It is eighty lines and is tested against the standard's known answers.

**Pack format, version 1.** A 32-byte header, assets on 16-byte boundaries in
`(kind, id)` order, and one table of contents at the end. No names, no times, nothing
compressed as a whole. Each entry carries a hash so that damage is found.

**Permanent IDs are hashes of keys.** 64-bit FNV-1a of the authored key. A collision
between two keys fails the bake and names both.

**Tiles are 64 m at level 0,** doubling each level. The Showcase's 500 m area is 8 by 8
tiles.

**The simulation steps 60 times a second,** with time accumulated in whole nanoseconds
and a stall clamped to five steps.

**Depth is reversed with an infinite far plane.** 1 at the near plane, 0 at infinity,
for even precision from arm's length to the horizon.

**The glyph atlas.** 128-texel cells, five levels, a distance range of 16 texels, both
faces in one array of 190 layers.

- IM Fell English is baked from its outline edges as a multi-channel field with the true
  distance in alpha.
- Tiny5 is drawn by its makers as many small squares that touch. Their shared edges are
  not edges of the letter, and a field made from them shows a grid inside every stroke.
  So the pixel face is filled first, on a grid four times finer, and its field is the
  exact distance to the edge of the filled shape.
- Glyphs are sized by the face's tallest and deepest character, so every letter shares a
  baseline, and centred by their inked box. A glyph wider than its cell allows is
  narrowed to fit.
- Small glyphs read the true distance, which survives being averaged into smaller
  levels. Large glyphs read the multi-channel distance, which keeps corners.

**The shimmer check measures flicker against a pixel's own trend.** For three
consecutive frames it takes how far the middle one is from the average of its
neighbours. The camera move must be slow, a small part of a pixel each frame, so that
steady motion measures as nothing. The brief's reprojection test can replace it once
there is depth to reproject with.

**Reference images are held to Mesa's software renderer only.** Cloud sessions and the
Linux CI runner both have it. Other machines round differently, and are not compared.

**Software rendering on the cloud machine** is Mesa llvmpipe through Vulkan (LLVM 20,
`deviceName = llvmpipe`). It supports pass timers, so the whole frame path is exercised
in the cloud. Its timings are never performance evidence.

**The performance run does not wait for the display.** It asks for an immediate present
mode so that frame time is the real cost of the frame. It sizes its window to 1920 by
1080 pixels whatever the display's density.

**The font half of S01-T02 was done in S00,** because the profiler view of S00-T15 is
drawn in the interface's own letters. The texture half stays in S01.

**The two typefaces were fetched without a separate approval.** The brief names both
files and their addresses and fixes their license, so there was no choice to approve.
Every other download waits for the owner's approval of `third_party/DOWNLOADS.md`.

**The macOS bundle identifier is `com.drodigital.vellmark`,** taken from the repository's
owner. It is the owner's to change before anything is published.

**Fonts are in plain git, not Git LFS.** They are 630 KB together. LFS is set up when
the first large pack source arrives.
