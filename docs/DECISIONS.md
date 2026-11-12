# Decisions

Newest first. One entry for each choice that is not already made by the brief.

## 2026-11-12: water (S02-T09)

**A pass of its own, after the opaque pass.** Water is drawn into the resolved scene,
reading a copy of it and its depth (one sample of the multisampled depth), so it can
see what lies beneath and behind it. It tests depth itself and needs no depth target.
It uses the terrain's grid: a tile with any water is drawn at the water's level, a dry
sample beside water takes its neighbours' level so the surface runs on into the bank
and the bank cuts it, and wherever no sample is wet the surface is not drawn. Dry is
stored as a level of minus a million rather than NaN, so the shader never tests NaN.

**The surface.** Ripples in three octaves, each fading before it is finer than a pixel,
carried by the flow in two layers that take turns, and drifted by the wind. The flow
runs down the water's own surface, faster where it is steeper; the pond is still.

**Light.** Fresnel between what lies beneath and what it mirrors. Beneath: the scene,
bent by the ripples (and not bent onto anything in front of the water), through peaty
water that takes red first and gives back a dark green-brown. Mirrored: a screen-space
march through the depth, 28 steps growing by a sixth each, and the sky where it finds
nothing or leaves the view. The sun's glint, in its shadow. The water thins to nothing
over the last three centimetres at the bank.

**Foam** where it is shallow at the edge and where it runs fast, broken by noise and
carried with the flow. Up close foam is written: `.` and `,`, and nothing between.

**Its letters.** Water has its own material, `water`, with the family `~ - = _` and its
lexicon (`water`, `H2O`). Its page drifts with the flow in the same two layers as the
ripples. The letters are written on the ripples, gathering on the crests and most of
all at the edge, so they never form a grid over open water. In the living look the eye
sees down into it.

**Not yet.** Rain rings (S02-T10); ripples where the player wades, and swimming (S03 on);
the brook's mouth into the river is carved as a steep dry-looking gully by the terrain
bake, which is the Content lane's to soften.

## 2026-11-12: grass (S02-T07)

**Patches as instances.** The ground near each view is cut into 4 m patches, and every
patch with grass is an instance in the one instance buffer, culled per view by the same
compute pass as everything else. A patch has no mesh: each blade is made in the vertex
shader from its number, on the terrain's own heights and ground, so grass costs no
memory beyond one shared list of indices. Grass grows on pasture and barley, a little on
a wood's floor, and nowhere too steep. The instance limit is now 8,192.

**Levels without popping.** Four levels of detail, each a quarter of the blades of the
last, chosen by the patch's nearest point to the nearest eye. Blades are placed along a
low-discrepancy sequence, so every level's blades are the first of the full set and are
spread evenly. Each blade fades by its own number as its distance grows: within 8 m the
grass is at 160 blades a square metre, beyond it the count falls with the square of the
distance and each blade widens to keep the cover, so a blade is about the same width on
screen at any distance and none is thinner than a pixel. Grass ends at 72 m, fading over
the last fifth, where the terrain's grass colour carries it.

**Bending.** Each blade is a curve from root to tip with its own lean, the wind's (gusts
that travel across the field, and a flutter), and a push away from anything within its
reach. Up to eight pushers; for now each view's eye, standing for the player. Barley
stands taller and stiffer, with a swelling head. The wind wanders on its own until the
weather drives it in S02-T10.

**Shading.** Blade normals are half the blade's own and half the ground's, so a field is
lit as the ground it covers and each blade still has its own light. Blades low in the
sward are darker, tips are paler, and light comes through a blade lit from behind.
Grass receives the sun's shadows but does not cast them yet; the shadow rota is S02-T11.

**A line of characters.** Each blade is written from root to tip with its lexicon's
words, a space between: `grass C6H12O6 grass C6H10O5`. Its cells are the blade's width,
so the text follows the same reveal rule as every surface, by the cell's size on screen,
and the glyph-off comparison holds. Every few seconds a blade moves on to its next word,
and now and then a letter flickers. At arm's length, where a cell is over about twenty
pixels, the blade between its letters thins away in the depth pass and the colour pass
alike: the blade is nothing but its characters.

## 2026-11-12: culling and instancing (S02-T06)

**One pool, one instance buffer.** Every mesh of a scene is in one vertex and one index
buffer; every placed thing is an instance in a storage buffer (transform, state, the
sphere that holds it, its batch), kept together by mesh so that each mesh is one batch.
Vertex shaders read their instance through a list, so the same draw serves every list.
Limits: 4,096 instances (now 8,192, with grass) and 256 meshes a scene. Grass, trees, and creatures will join it.

**Lists.** List 0 is every instance, in order, for the shadow maps, which are not culled
yet (their far cascade's rota is S02-T11). List 1 + v is what view v can see, written by
the culling pass each frame, with an indirect draw per batch whose count it adds up.

**Culling** is a compute pass before the depth pre-pass: each instance's sphere against
the view's frustum (four sides and the near plane; the far plane is at infinity), then
the box around it against last frame's depth pyramid at the level where it covers two
texels or fewer. Depth is reversed, so a thing is hidden when its nearest depth is less
than the least depth drawn over its box.

**The pyramid** is built right after the depth pre-pass, from one sample of the
multisampled depth, at half resolution, each level the least of the four beneath it
(three where a level is odd, so nothing at an edge is lost). It is read next frame.

**Never trusted after a cut.** If the eye has moved a metre or turned more than about
14 degrees since the last frame, or the targets were made anew, a view culls by its
frustum alone that frame. So something that comes into sight is never a frame late.

**Proof.** The reference images, the shimmer shots, and every capture are unchanged with
culling on; a test with its own scene shows a crate behind a wall culled once the pyramid
holds the wall, the crate beside it never culled, everything behind the eye culled, and
the hidden crate drawn the moment a cut brings it into sight.

## 2026-11-12: the sky each place sees (S02-T05)

**Terrain:** the bake finds the horizon in eight directions from every sample, out to
about 120 samples in growing steps, and stores the cosine-weighted share of the sky seen
(`1 - sin²` of each horizon's angle, averaged) as a byte per sample (`VLMKTER2`).

**Meshes:** box faces are divided into cells of half a metre, so the value can change
across a face, and each vertex casts 48 rays spread by the cosine over its hemisphere, to
25 m, against every box of the scene, still and moving (as the box that bounds each
where it stands). The share that escapes is stored per vertex (`VLMKMSH2`, 64 bytes).

**Use.** The sky's light on a surface is scaled by its share. The sunlit ground's bounce
is not: a place that sees less sky sees more ground. The bounce was weak; it is now a
tenth of the sun's light on the ground, about what a pale floor returns.

**Contact occlusion** is a compute pass after the depth pre-pass, at half resolution,
reading one sample of the multisampled depth: eight neighbours on a spiral about 35 cm
across, counting what rises in front of the surface within half a metre. It darkens only
the sky's light and the bounce. Its texture is sized for half of 4K and each view writes
its own part, so the frame's bind group never needs remaking. The shimmer check passes.

**Not used yet:** the sky share for fog's colour, rain, and snow, as `docs/TECH.md` plans.

## 2026-11-11: the air (S02-T04)

**A froxel volume per view:** 160 × 90 × 64 cells, slices spaced exponentially from
0.5 m to 400 m. A compute pass lights each cell's mist (sun through the shadow maps with
a forward-scattering phase of g 0.55, and the sky's light) and a second sums each column
front to back, energy-conserving within each cell. Surfaces and the sky read the sum with
one filtered sample. Its strength by the hour is in the place's grade (`VLMKGRD2`): thick
in the river's hollow at dawn, nearly gone at noon.

**God rays** are the shadow maps inside the mist. The last cascade now reaches 260 m, so
that the hills shadow it across the valley. Their shafts are faint on bare hills; trees
(S02-T08) and buildings (S06) will cut sharper ones. Nothing more is needed in the fog.

**Aerial perspective** beyond the mist is the atmosphere's own extinction over the
distance at the path's mean height, with the horizon's light filling in: far hills go
blue and pale.

**Shade.** Single scattering leaves out most of the light in shade, so the sky's light on
surfaces is raised by half (`MULTIPLE`), and the sunlit ground sends a little back.
The yard's reference images were accepted again for these and for the mist.

**The fog's shaders are made at startup only:** a changed `fog.wgsl` needs a restart, not
a hot reload.

## 2026-11-11: the sky (S02-T03)

**One atmosphere, twice.** Single scattering by air (Rayleigh), haze (Mie, g 0.8), and
ozone, on a round earth: in `crates/render/src/sky.rs` for the sun's and moon's colour
where they reach the valley and the sky's light on surfaces, and in `shaders/frame.wgsl`
per sky pixel. The same constants in both, so the light on a wall at dusk is the light
of the sky above it. Per pixel it is 12 steps along the view and 6 towards the light:
about 72 samples on sky pixels only. If that is over budget on the owner's Mac, the
answer is a sky-view table drawn once a frame at low resolution (the usual fix).

**Units.** The sun is 6.4 of the renderer's light at the top of the air; the full moon a
twentieth of that, far above the real one-in-400,000, because the game must be playable
at night; starlight and airglow are a fixed faint blue. Exposure moves from night (1.8)
to day (0.62) over the sun's last few degrees, so dawn is not seen with night's eyes.

**The moon** keeps ten hours behind the sun, so it is a waxing gibbous moon up in the
evening; its phase is lit from the sun's true direction. Its seas are the `@` of the
printed face, read from the glyph atlas at a coarse level and only a fifth as dark as the
rest, so they suggest the letter and do not spell it. It is drawn the right way round.

**Stars** are one candidate in each cell of a 900-row grid on the sphere, a few in a
hundred lit, turning about a pole 52 degrees up in the north, faded as the sky brightens
and dimmed near the horizon by the air.

**Clouds** are two flat layers, broken cumulus at 1.8 km and a thin veil at 8.5 km,
drifting, self-shadowed towards the light, and fading into the horizon's haze. Cover is
0.38 until the weather (S02-T10) drives it. They cast no shadow on the ground yet.

**The colour grade** is data, `assets/grades/vale.ron`, keys by the hour blended round
the clock: lift, gamma, gain, saturation, after the filmic curve. Shadows lean
slate-blue as the canon's palette asks. A tool's mask skips the grade: graded, black is
no longer black, and the glyph-off comparison would have had nothing to compare.

**The yard's reference images were accepted again,** because every pixel's light changed.
The glyph-off comparison and the shimmer check pass as before.

## 2026-11-11: drawing the terrain (S02-T02)

**One shared grid, heights in a texture.** Every tile is drawn from one index buffer of
64 × 64 cells; the vertex shader reads the tile's heights from a layer of an array
texture. The heights carry a border sample from each neighbour, so normals agree across
seams. One draw call a pass draws all 192 tiles as instances. Culling comes in S02-T06.

**Rings cut by a test, not by geometry.** A coarse ring discards fragments inside the
next finer ring, in the depth pre-pass, the lit pass, and the shadow maps. The rings'
borders lie on the coarse grid's lines, so no cell is ever half in.

**Ground blends two materials at a time,** the strongest two of the four samples' classes
and rock, by their weights and their own height maps. Steep ground (normal below about
55 degrees from vertical) and the highest ground are rock; roads, river beds, and the
Erasure stay what they are however they lie.

**Letters are projected per 1 m cell,** along the axis the cell's own normal most faces,
chosen hard and never blended, as rule 3.1 asks. The letters are those of whichever of
the two materials shows at the pixel.

**The rules stand players on the ground** through a `Ground` trait the world crate
answers from the finest tile. Steps, slopes, and falling come with the character (S03).

**The hill lane is as steep as the hill,** about one in three. A gentler limit built it up
into a dyke several metres high.

## 2026-11-11: the living lattice (look 4)

**A separate function, not a mode of the old one.** `living_lattice` follows the same
world-fixed planes as `lattice`, so the earlier looks stay as they were for comparison.

**Colour from the material at the character's own place.** The hit point is carried
back into the page's anchor frame through the surface's tangent and bitangent, and the
material's colour is read there at mip 3 (one texel about a character), shifted a
little with depth so each layer is a different slice. The light is the surface's own
(lit colour over albedo), dying away as `exp(-7 × depth)`, plus a faint glow of the
material's hue divided by exposure, so the inside can be seen at night.

**The surface opens by letters, not by a shape.** Each front cell has its own threshold
and fades over a band, so nothing pops and no outline is ever drawn.

**Fewer planes, stepped.** Four planes per family, each 0.72 of the last's brightness,
read as layers; seven read as static.

**No character is mirrored.** Which families of planes are seen from behind is known
from the ray's sign; their characters and words are turned back.

**Names and elements in the fine print use an ordered pattern.** The surface is ruled
into patches twenty cells by three rows, offset half a patch row by row; each patch
shows in its turn by a 4 × 4 Bayer order, so the words shown at any moment are evenly
spread. The owner asked twice for them in the grain's own style (not brighter, not
bolder, not set apart) and more often: 0.6 of patches from a step back, 0.3 at arm's
length.

**One file of words for every material.** `lexicon.ron` is keyed like `materials.ron`;
the bake joins them and fails if a material has no words. The brief's table is that
file written out, and a test compares them.

## 2026-11-11: the terrain (S02-T01)

**Three nested grids, each one level of the tile layout.** The playable 512 m at a
metre (level 0), the far ring to 2 km at 8 m (level 3), and the horizon to 8 km at 32 m
(level 5), each 513 samples square and cut into 8 × 8 tiles of 65. Format `VLMKTER1`:
heights, the water's surface (NaN where dry), and a ground class for each sample.

**One function shapes the land at every level,** with as many octaves of noise as the
grid can carry, so the levels agree. Then erosion (droplets, then slumping), then the
landmarks from `assets/terrain/valley.ron`. A finer grid eases into the coarser one over
28 of its samples at its border, so that along the border it lies exactly on the coarser
ground, and is then written into the coarser grid wherever both have a sample.

**Water finds its own level** from the land: a running minimum down the river, a little
below the ground, so it never runs uphill and moving a river never floods a field. A
bank holds the level of the river a little upstream, so a bend cannot let water out.

**Where the water check is not held.** Each region is checked on its own grid, except in
each grid's outer seam band, where a river is carved at two widths as one grid eases into
the next. In the playable area that band is the last 28 m, beyond the steep edges; the
largest spill found there is 8 cm, where the river leaves through its gorge.

**Roads have a steepest grade.** Where the land is steeper the road is cut in or banked
up. Where the road meets the brook it dips to the water, and the footbridge (S06) spans it.

**Generated, not hand-modelled, and cached.** The bake takes about five seconds with
optimisation, so the bake crate is always built optimised, and the tiles are cached by
the hash of the file and the generator's version.

## 2026-11-11: loose letters

**Letters in flight against the world and the heaps; heaps as a height map.** Contact
between every pair of tens of thousands of bodies would cost a broad phase every step,
and in a spill nothing shows two letters in flight striking. What does show is letters
lying on letters, so the letters at rest are kept as a map of heap tops, one cell a
letter wide (1.28 cm), each cell holding a heap for each surface it stands on (under a
bench and on it). A letter landing on a heap lies on its top. Before it lies down it
slides to the lowest neighbouring cell while that cell is lower than the angle of repose
allows (0.7 of a cell, about 35 degrees, which is how steep flat flakes stand), which is
the sandpile rule, and gives true cones and ridges. It keeps its place within its cell
as it slides, so a heap is never laid out on a grid. Each letter adds 3 mm: thicker than
a letter looks, so that a broken door still makes a heap of a door's size, and thin
enough that layers seen edge-on do not gape. When letters at rest are swept, the heaps
are measured again from the letters left.

**Shadows of loose letters are their squares, drawn smaller.** The cascades are too
coarse to carry a 1.28 cm character's shape, and a whole square shades like a tile; a
square of 0.62 of the side matches the ink a character covers.

**A letter pushed out of a box leaves by the nearest face that opens into free space.**
The first version took the nearest face, and a letter in the corner where the doorstep
meets the wall was handed between them and came to rest inside the wall.

**The scene keeps its boxes for the rules.** `VLMKSCN2` adds the static boxes as
axis-aligned solids, and each object's boxes as its breakable parts. A turned box is given
the box that bounds it, which is exact for everything in the yard.

**Drawn as instances, sent only when they move.** One instance a letter (position, turn,
material, seed); the buffer is rewritten only on the steps where letters moved, so a
settled spill costs only its drawing. They are drawn after the opaque surfaces in the same
pass, writing depth, with alpha to coverage cutting out the character. They are not in the
shadow maps yet.

**Deterministic.** The burst uses the rules' own random numbers, and the world's hash
covers every letter, so a break replays exactly.

## 2026-11-11: the depth looks

**The lattice is three families of planes, not a volume march.** Planes parallel to the
surface carry the edges that run across and up the page; planes standing on the page
carry the edges that run into it. A pixel's ray is followed through at most seven planes
of each family, a pitch of six cells (7.7 cm) apart, and on each it lands on a character
or on nothing. The parallax and the perspective are exact, because each pixel uses its
own ray.

**No line is drawn.** Only cells on the lattice's edges hold characters. Crossings are
drawn brighter, since they are what the eye lines up into a frame.

**The lattice is fixed in the world, not in any surface.** It was first laid out on each
surface's own page, so it was cut off wherever one thing met another. It is now one
cubic grid along north, east, and up, and every surface is a window onto it: a wall, the
frame of the door in it, and the ground at its foot all show the same grid, unbroken
where they meet. Which character stands in a cell no longer depends on the material in
front of it, only which alphabet it is drawn from. Its fog is measured in metres, not in
shares of a thing's thickness, so a thin thing beside a thick one shows no seam, and a
thin thing is seen into by at least a quarter of a metre.

**The eye sees further in the closer it comes.** The fog between the letters thins with
nearness: from two metres only the first centimetres can be made out, at arm's length
the far side of a wall.

**Nothing in a depth look is anything but a character.** The texture was first taken
away with an ordered dither, whose dots are small squares, and the lattice's edges were
solid rows of characters that read as bars. Both were geometry that is not writing, and
the owner asked for it gone. Now the texture retreats into the strokes of the surface's
own fine print, and along a lattice edge every other cell is empty, so each character
stands alone and the grid is only their alignment.

**The lattice is written in the thing's own letters, and lit only by depth.** A version
ruled in ASCII's special characters, with a pulse, a colour shift with depth, glowing
crossings, and loose drifting characters, was built and shown; the owner preferred the
plainer lattice, and those were taken out again. The nested frames stay: an edge of a
larger frame is brighter. Deeper is dimmer, and that is all.

**Written words are given thickness.** A written letter is drawn again four times
behind itself across a fifth of its cell, in shade, so that from the side it is a small
block. The lattice is hushed behind a passage, easing off over one cell at its edges.

**A passage sits on a quiet plate.** The bake clears the grain from the rows a text
uses, the blank rows between its lines, and a row of margin, and a text may ask for a
blank row between lines (`leading: 2`). In the flat look a written cell is drawn 1.7
times firmer than grain, and the finer grain under it is quietened to 0.3.

**Thickness is baked for every vertex** as the box's extent along the face's normal. The
lattice reaches that far and stops.

## 2026-11-11: Phase S01

**The yard's materials are generated until the textures are approved.** The brief allows
content generated by code. The generators live in `bake/src/materials.rs` and produce the
same asset the CC0 maps will, so the switch is a change of data.

**One vertex format, 56 bytes:** position, normal, tangent, anchor, material, text page.
Everything static in a scene is one mesh and one draw; things that can move are objects
with a transform.

**Pages are world-aligned.** A box's faces are laid on the grid of the space it was
authored in, reading left to right seen from outside, so text runs unbroken across
boxes that share a plane. A turned box keeps the page it would have had unturned.

**How glyph_surface() keeps the two promises.**

- *Size on screen* is a cell's side divided by the page's greatest stretch across a
  pixel. That is the cell's smaller dimension on screen, so a surface at a grazing
  angle loses its letters before they can alias.
- *The early exit* is taken when no allowed level reaches 3 pixels. Beyond the reveal
  distance the shader samples nothing, and the frame is bit for bit the frame with the
  layer off. The capture tool proves this with a reveal mask.
- *Tone.* A level multiplies colour by `1 + contrast * (mean - coverage)`. Its mean over
  a cell is exactly 1 when `mean` is the glyph's true mean coverage. Sharp glyphs take
  that from the baked lookup. A glyph under about 8 pixels is drawn so softly that its
  coverage is linear in distance, and there the mean follows from the glyph's baked mean
  distance. Measured: the letters move mean brightness by under 2%.
- *Weight* comes from the surface's own colour, never from light, except in close
  reading, where the brief asks for light to become ink weight.
- *Letters made of letters.* A cell's target weight is raised or lowered by its parent
  glyph's coverage at the cell's centre. Each level is four times the last, so a parent
  is a 4 by 4 mosaic of children; whole children inside a parent's stroke would need
  16 to 1. Two levels apart, the relation is 16 to 1 and reads as the brief describes.
- *Two faces.* The pixel face below about 9 pixels, the printed face above about 12.
  The band is narrow because two faces drawn half-and-half read as a blur.
- *Contrast grows with nearness:* 0.7 of a material's contrast where letters first show,
  1.7 at arm's length.

**The reveal distance scales with the view's height,** as the brief says, so a 270-pixel
capture reveals at the same distances as a 1080-pixel game. Letters in a small capture
are therefore a quarter the size in pixels; legibility is judged from 1080p crops.

**The dive stops at level -4** (25 micron cells). Below that, single-precision anchors
can no longer place a cell. A dive ends by cutting to the next scene, as the title
sequence will.

**Authored text is laid out by the bake.** A text names tagged boxes and a facing. Words
are set left to right, top to bottom, never split, and skip any cell that is not wholly
on the surface, so text flows around a doorway. A cell across the seam of two coplanar
faces counts as on the surface. Pages live on one 1024 by 1024 sheet for each scene.

**Lights.** Point lights are sorted on the CPU into 16 by 9 by 16 view clusters, 64 bits
to a cluster. The shadow-casting light nearest the first view gets a 512-pixel cube.
Glowing materials cast no shadow, or a lantern's glass would shut its flame in.

**The edge filter is switched off where letters are drawn.** The lit pass writes the
reveal mask into alpha, and the filter's strength is one minus it. Type stays crisp.

**Resolve is the hardware's, in HDR.** Bright edges may alias more than a tone-mapped
resolve would allow. Revisit if the Look Gate finds it.

**Exposure follows the hour, not the picture.** There is no eye adaptation yet.

**Close reading's paper and ink are divided by exposure,** so paper is the same paper at
noon and at night.

**The determinism test is exact on the software renderer and tolerant on hardware.**
The macOS CI runner's GPU gave two runs that differ in last bits. Hardware is held to
3 steps of 255 in any pixel and 0.0005 mean, and the test prints what it measured.

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
