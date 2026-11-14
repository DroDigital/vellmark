# Decisions

Newest first. One entry for each choice that is not already made by the brief.

## 2026-11-14: words on the weapon, combat lines, and the bow (S03-T10)

- **On the weapon.** Both hands together, weapon and word, write the word on the weapon at once, with no flight, for the word's Ink (half again at full charge). It lasts 30 s, 60 s fully charged; a typo typed as it charges does the typo's `weapon` result. What a weapon carries passes to what it strikes: `ignite` for 5 s, `frost` as a 4 s slow. A sharp edge sets armour aside and cuts 15% deeper.
- **Combat lines as marks.** Each word's effect on a creature is read by steel as a mark: gaping (blows do half again), slowed (a heavy blow shatters it for 60% more and ends it), burning (4 a second at full strength), pacified and dazzled (for the Inkling's mind).
- **The bow.** Full draw in 0.9 s; an arrow leaves at 18 to 60 m/s with 6 to 32 damage by the draw, under gravity; a held draw drains 7 Stamina a second and looses on its own when Stamina runs out. A quiver holds 20; stuck arrows within 2.2 m of the eye are gathered with Interact.

## 2026-11-14: steel and Stamina (S03-T09)

- **Numbers.** Health and Stamina 100. Stamina comes back at 28 a second once nothing has spent it for 0.8 s. A sidestep costs 18, a shield bash 16, a sword's heavy attack 24. Close reading drains 9 a second (4.5 in co-op). Sprinting is free: the brief names what Stamina pays for, and running is not among them.
- **The chain** is three swings, right, left, right; a tap within 0.45 s of a swing ending carries it on, and a tap during a swing is kept for when it ends. A swing strikes between a third and three fifths of the way through. A heavy attack is drawn back while held and released on letting go; too tired for it, it is a light swing.
- **Blocking** takes 60% of a blow faced within about 70 degrees with a weapon and 85% with a shield, at 0.9 Stamina a point kept; with too little Stamina the guard breaks and the player staggers 0.8 s. A blow on a player that gets more than half through interrupts the hands, as a blow did in S03-T07.
- **Weapons by moveset.** Base numbers per type are in `melee.rs` until the loot generator of the full game gives each weapon its own; the trait of each is as `5d-loot.md` names it. A staggered creature takes a quarter more.
- **Hit stop** holds the striker's view on the frame of the blow for 55 ms (110 ms for a heavy or sneak blow) and kicks it slightly; it is the app's, per view, and never the rules'.

## 2026-11-14: props, breaking, and the Mended Bridge (S03-T08)

- **Letters are the pieces.** The brief pre-fractures breakable props into pieces that fall under physics and crumble into characters within seconds; here a broken thing goes straight to its letters, which already fall and settle with their own physics (`docs/PLAN_CHANGES.md`, 2026-11-11). A marked structure's pieces are its authored boxes, drawn one by one (`pieces: true` in a scene), so that `WHOLE` puts it back a piece at a time.
- **Mending.** Each letter flies home along an arc from where it lies, which is the parabola it fell along when it fell freely; letters set out so that neighbours arrive together, and later the further their home is from the writer's feet (0.11 s a metre). A piece is drawn and bears weight the moment its last letter is home; the thing is whole, and its Inscription says `WHOLE`, when every letter is. Mending lasts until something breaks it again. `HOLE`, the typo, knocks a whole breakable thing down.
- **A budget per thing.** One thing falls into at most 24,000 letters (a door's worth). A bigger thing scatters its letters over its faces rather than through its volume, since its print is what is seen.
- **Props.** Loose props are rigid bodies in their own `rapier3d` world, built on the same still things as the body's, settled at load. Interact picks up a prop of up to 40 kg within 2.6 m; it is drawn 1.3 m before the eye and let go if it catches on something; the main hand throws it at 11 m/s for 5 kg, less for heavier things. A breakable prop whose speed changes by more than 7.5 m/s in one step breaks. Things burning longer than 15 s break.
- **Not yet:** props neither block the body nor are pushed by it, and do not stand on mended structures. Their weight words (`HEAVY`, `LIGHT`) come with the full game.

## 2026-11-13: Inscriptions and casting (S03-T05)

**Numbers, for the Play Gate.** A charge is full in 1.2 s. Tough targets need half a
charge, Hard a full one; both hands holding the same word reach a tier further, half again
as strong, and cost 1.6 times as much. A staggered or unaware target resists a tier less.
Written words hold 45 s from a tap and up to 120 s from a full charge, half as long again
from both hands, then fade back to what they replaced. Ink: 100, refilling at 2.5 a second
while no hand charges; a word costs its sheet's Ink, more on larger things (0.6 to 3
times by size), on tougher ones (up to twice), and with its charge (up to twice). Words
reach 40 m.

**A word joins or replaces.** A word replaces the word of its class on a target; if there
is none it joins the Inscription, up to three words, a written one giving way when full.
So `BURNING` on the yard's `LOCKED` door gives `LOCKED BURNING`, and the door burns locked.

## 2026-11-13: the word sheets (S03-T04)

**Thirteen target classes.** The brief asks for a result or a refusal for every word on
every target class but does not list the classes. The Showcase's are door, container,
lamp, fire, water, structure, plant, creature, inkling, person, weapon, paper, and stone,
in `assets/data/words/targets.ron`; the full game adds to the list, and every sheet must
then answer the new class before the bake passes.

**Sheets name effects; the rules carry them out.** A sheet says which effect a word has
on a class (`open`, `ignite`, `freeze`, and 33 more, listed in `bake::words::EFFECTS`),
how strongly, and the preview; the rules hold the effects' mechanics. The preview is the
effect's own (`preview.open`, "Opens it") unless the sheet names another. People refuse
harm and Mind words other than `CALM`; `OPEN` and `HOLE` on a person refuse with the
canon's "Not without a signature."

**Motifs in D Dorian.** `OPEN` D A D, `LIT` A C E G, `WHOLE` D F E D, `CALM` A G E D,
`BURNING` E F E B C, `COLD` B F B (falling tritones), `SHARP` C G F. The Vale's four
Standing Page words (`OPEN`, `LIT`, `WHOLE`, `CALM`) will make the main theme. The bake
refuses two motifs with the same intervals.

## 2026-11-13: the hands and the settings (S03-T02)

**The rules read the hands.** Which hand tapped, held, or let go is worked out in `sim`
from the buttons each step, so bots and replays see exactly what a player's hands did:
a press let go within 0.22 s is a tap; held longer it is a hold that charges until let
go; the second hand pressed within 0.12 s of the first, before either is a hold, makes
both hands act as one until either is let go.

**Settings are multipliers and differences.** The settings file holds sensitivity as a
multiple of the default and only the buttons the player moved, so a later change of a
default reaches every player. Moving an action onto a button swaps whatever was there
onto the action's old button: nothing is left without one. Prompts are string keys
(`prompt.key.space`, `prompt.pad.playstation.south`), so their letters are in the
string table, and every key the code can build is checked to be there.

## 2026-11-13: the body (S03-T01)

**Rapier for the still world; the body's own state for the rest.** The terrain's finest
tiles become heightfields, each cell split from its least corner to its greatest as the
terrain is drawn (rapier's zigzag split), so feet stand exactly on the drawn ground. The
scene's solid boxes are cuboids and each tree's trunk a capsule as thick as its record
says. This world never changes once loaded, so it is shared between copies of the
`World`. A player is a capsule moved by rapier's kinematic character controller, with
autostep to 0.38 m and snapping to the ground; jumping, the sidestep, the mantle,
crouching, and swimming are the body's own small state, stepped at the fixed rate, so
the same inputs give the same body. `enhanced-determinism` is on.

**Numbers.** Jog 5 m/s and sprint 7.5 m/s (the brief's), crouch 2.4, swim 2.6 and 3.8
sprinting. Gravity 20 m/s² and a jump of 6.4 m/s, about a metre, with 0.12 s of grace
before landing and after leaving an edge. The sidestep is 2 m in 0.22 s, fast out and
easing in, untouchable for its first 0.12 s, with 0.18 s before the next. A ledge from
0.45 m to 1.9 m above the feet is mantled in 0.42 s when the body jumps at it or pushes at
it in the air. Water deeper than 1.25 m is swum, the eye 0.18 m above it; shallower water
slows the legs. All of it is feel, for the owner's hands at the Play Gate.

## 2026-11-13: the ambience is synthesized until the samples come (S02-T13)

**Our own mixer, on SDL3's audio stream.** `vellmark-audio` holds a mixer that the
platform crate's SDL3 playback stream calls on the audio thread. It reads what the game
wants through atomics and renders into a buffer made once: nothing is allocated or
locked there, and a test holds it to that. No `fundsp`: the few building blocks needed
(pink noise, a state-variable filter, a sine, smoothing) are a hundred lines.

**Four layers, all chance.** Wind is band-passed noise whose band rises with gusts that
come and go every one to four seconds, a low roll under it and a whistle in a strong
gust; leaves hiss and flutter in it near trees. Rain is a hiss and drops striking at
random pitches and places, more of them as it rains harder. The river is a low run of
water and resonances that jump about as it turns over stones, louder as it is nearer.
Birds are calls of two to six notes from three kinds of bird, falling, rising, or
trilled, by day, in the open, and not in rain or a hard wind. The game sets them from
the weather, the hour, water found in rings about the listener, and the trees near.
Recorded beds from the approved packs go under them when the owner fetches them
(`third_party/DOWNLOADS.md`).

## 2026-11-13: a tree's bark records its age (S02-T08d)

**The truth is data.** The bake records each placed tree's kind, age, and girth in a
`trees.records` table beside the scene. A tree's age is its kind's age at its grown size
(`age` in `assets/trees/kinds.ron`: oak 140, ash 90, apple 40, hawthorn 55), older as the
tree is larger (scale to the power 1.6), and up to 15% older or younger by where it
stands. Nothing about it is written into the pack as text.

**The text is written where it is read.** The page table keeps its last eight pages, and
the page sheet its bottom 128 rows, for generated truth; authored text has the rest, and
the bake refuses more. The renderer lends the eight pages to the trees nearest any eye,
within 14 m, and rewrites a page only when the tree it is lent to changes. A tree's
instance tells its bark which page to read, and any surface of it without an authored
page reads it, its leaves never. The record runs round the trunk from 0.35 m to 2.75 m
up, at the second level of the cascade (letters about 2.5 cm tall), a blank row between
lines and each line a little further round than the last. Its words are in the string
table (`assets/text/en/trees.ron`): `Oak. Stood 130 years.`, after the yard's bench.

## 2026-11-13: grass about a pixel wide is shaded as the sward (S02-T07b)

**Prefiltered, not blurred.** Pasture seen almost edge on, and barley against a low sun,
failed the shimmer check: a blade a pixel or two wide carried detail finer than the
pixels (its root's shadow, its rounded and randomly turned face, a lit tip over a shaded
root), and 4x MSAA turned it on and off as the eye moved. The brief allows no temporal
accumulation, so the detail is filtered where it is made. A blade's `seen`, from 0 at
about a pixel and a half wide to 1 at four, blends it towards the sward's average:
the sward's normal (90%), the root's shadow at its mean, the sward's share of the sky
(55% of the root's), and the sun's shadow taken partway up the sward (0.15 m in pasture,
0.6 m in barley), where the ground under it takes it too. Each blade keeps its own tint
and its paler tip. Against a low sun the ground under a sward glows as its blades do.
Thin blades are darkened to 0.8 so the field keeps its brightness: the near pasture's
mean colour is within 1% of what it was, and its contrast from pixel to pixel falls
from 0.080 to 0.061. Evidence: `docs/evidence/S02/grass_grazing.jpg`.

**The barley's shimmer shots moved into the field.** The road's hedges, placed in
S02-T08b, stand where S02-T14 took them from (-60, 44); they are now taken from inside
the barley at (-50, -20). Three grazing pasture shots were added to the exit captures.

## 2026-11-13: trees drawn (S02-T08c)

**The level is chosen once, in culling.** Each tree's level comes from the share of the
view its bounding sphere fills (`shaders/lod.wgsl`: below 9%, 4%, and 1.8% it goes one
level coarser), and the culling pass writes it into that level's batch. A view's list
keeps a place for every level of every tree, after the scene's own instances; the grass
follows them. The shadows draw each tree once, from its third level.

**Crossfades and cut-outs over the pixel's samples, not by discarding pixels.** Over the
fifth above each share, a tree is in both levels' batches and the two share out the
pixel's four samples, the coarser taking a run and the finer the rest. Leaf cards cover
the samples their leaves do, and the far form those its picture does, with leaves too
small to tell apart seen as their average cover. Whole-pixel screen doors were tried
first; they crawl under a slow move. The depth pre-pass and the lit pass compute the
same samples, so the pre-pass's equal test still holds.

**Leaf cards are two-sided and never mirrored.** From behind, a card's anchor runs back
along it and its normal and tangent turn, so its `^ A v` read the right way round. A card
seen edge on fades out. Each card's side is carried as its vertices' thickness, and its
number in its anchor (cards a hundred metres apart), so the shader cuts the same spray
the bake painted into the far form's picture.

## 2026-11-13: trees grown in the bake (S02-T08a)

**Grown, not modelled.** The kinds are data (`assets/trees/kinds.ron`: oak, ash, apple,
hawthorn), each grown from its seed by `bake::trees` the same way every time: a leaning
trunk, branches leaving it in golden-angle turns above the crown line, each branch
branching again, drooping under its weight and reaching up at its ends; cards of leaves
at the ends of the finest branches, facing out from the crown with some scatter.
Approved model packs (Quaternius in the catalogue) wait for the owner's download list.

**Bark carries a page.** Each branch is a tube whose page runs round it and up it, with
one seam, so its letters wrap the trunk. Its material is `bark` (`| ! 1 l`, and its
lexicon). Leaves are `leaves` (`^ A v`). Cards are squares; the shader cuts each into a
spray of leaf ovals (`trees::leafy`, to be mirrored in the shader in S02-T08c).

**Levels.** Three levels and a far form. Each level draws fewer, coarser tubes (9, 6,
and 4 sides for the trunk; twigs dropped) and one card in 1, 2, then 5, made larger by
the square root of what was dropped so the crown keeps its cover. An oak is 5,886, 1,684,
and 556 triangles.

**The far form keeps the true silhouette.** Two crossed cards as wide as the crown and as
tall as the tree, in an impostor material per kind (`impostor.oak` and so on), made in
the material bake by projecting the finest level from the side: bark first, leaves over
it, each card cut to its leaves, coverage in the height channel. Far forms have no words.

## 2026-11-13: every weather, and the shimmer across the barley (S02-T14)

**Fog is weather.** The rules keep a fog amount beside the rain; it gathers and lifts at
a quarter of the rain's pace and stills the wind. It adds 0.012 a metre at full to the
hour's own mist, raises how high the mist reaches, and dims the sun by up to 60%.
Console: `fog 0..1`.

**The exit captures** are `captures/s02_weathers.ron`: the valley from Tithe Hill at
dawn (6:24), noon, dusk (17:48, the hour of the golden barley; 18:18 is already night
in this sky), night, rain, and fog, and the shimmer check across the barley in clear
air, at dusk, in rain, and in fog. Every shot is also a glyph-off comparison.

**A shimmer run moves the eye 12 mm over 24 frames.** The check assumes the picture
moves less than a pixel a frame; grass a metre from the eye moves four pixels a frame
at the yard's runs' speed, which is motion, not shimmer.

**What the check found, and what fixed it:**

- Grass blades are never drawn thinner than about a pixel, tips included. An edge finer
  than a pixel switches its samples on and off as it moves.
- Wet blades shine broadly (roughness no less than 0.55), and the ground under a sward
  is matte where blades are drawn: sharp sun glints on narrow swaying blades, and on
  ground blinking between them, sparkle.
- Wet surfaces gloss to roughness 0.28 at most; only standing water is a mirror.
- A puddle's edge is softened to the pixel's footprint, since multisampling does not
  smooth an edge drawn by a shader.
- Rain streaks fade in and out along their length, so a pixel brightens and dims as a
  streak passes instead of switching on.
- Blade words crossfade when a blade moves on to its next word, and the instant flicker
  of single letters is gone.

**Measured:** p99 0.0176 clear, 0.0113 dusk, 0.0175 rain, 0.0192 fog, all under 0.020.
Wet ground in full sun after rain, which is not one of the gate's weathers, measures
0.0214; it is listed in `docs/STATE.md`.

## 2026-11-12: the governor and the presets (S02-T12)

**One governor** (`render::governor`), fed every frame the sum of the GPU's pass timers
(the frame's wall time where the GPU cannot time its passes), smoothed over half a
second. Above 16.0 ms it gives up a tenth of the next thing in the brief's order and
waits a quarter of a second; below 14.1 ms it takes back a tenth of the last thing
given and waits three quarters. Between the two it holds still. The timers are now on
in every build where the GPU has them, since the governor reads them.

**What each step gives up**, from nothing to all of it:

| Knob | At nothing | At all |
|---|---|---|
| The far cascade | redrawn after a quarter of a degree | after a degree, and at most every eighth frame |
| Particles | all | half |
| Grass beyond 30 m | all | 35% |
| The fog grid | 160 by 90 by 64 | 60% along each side, a sub-grid of the same texture |
| Contact occlusion | full | none, and not computed |
| The scene's resolution | 100% | 85%, drawn in the corner of full-size targets and stretched with a sharpening filter |

Nothing is reallocated as the knobs move, so nothing stalls; letters keep their reveal
distances because the reveal is measured against the scene's own size. Reading brings
the resolution back at once. Each step is logged, and the performance report lists
them with the lowest resolution reached.

**Presets.** Low, Medium, and High set grass density and its near distance (80, 120, and
160 blades a square metre within 6, 7, and 8 m), how many lights cast shadows (1, 2, 3),
and the share of particles (60%, 80%, all). Medium is the Showcase's target and the
game's default; `--preset` and the console's `preset` choose another. Captures and
tests draw at High with nothing given up, so the evidence does not move. Choosing Low
automatically on the minimum machine waits for its numbers.

## 2026-11-12: cached shadows and rationed lights (S02-T11)

**The far cascade is kept.** The two near cascades are drawn every frame. The far one,
out to 260 m, is fitted to a sphere around a cell of a grid an eighth of its reach,
not to the slice of the view, so turning the head never changes it. It is drawn once
and kept until the light has moved a quarter of a degree (about two seconds of the
day's clock), the eye has crossed into another cell, or a still thing has changed:
the scene, the terrain, or what is broken. Its texels are a little coarser than a fit
to the view's slice (about 29 cm against 15 to 25), which at that distance is not seen.

**Shadowed lights are rationed.** The three nearest lights that cast shadows each hold a
cube of a cube array. A cube is drawn once and kept while its light and what casts
shadows near it stay as they are, loose letters included, and at most one cube is
drawn in a frame. A light newly among the nearest lights without a shadow until its
cube is drawn, which is the next frame or two. The count is the presets' to change.

**Cube arrays** are core WebGPU and present on Metal, Vulkan, and DirectX 12; the build's
shader check now allows them.

**Not yet.** Grass casts no shadow: its blades are made from the view's eye, and the
near cascade would need its own pass of them. Judged with the governor's numbers.

## 2026-11-12: rain and the wind that reads (S02-T10)

**Weather is the rules'.** `sim::weather` keeps the rain, how wet the world is, and the
wind, steps them with the world's clock, and sends them in the snapshot, so every view
sees one sky. Rain comes and eases at 8% a second; a downpour soaks the ground in about
a minute, and it dries in eight. The wind wanders by the clock and blows harder in rain.

**The sky in rain.** The hour's atmosphere is closed over: cloud cover rises, the sun is
lost behind it (85% at full rain), the sky's light greys and dims, and the mist thickens.

**Wet surfaces.** What sees the sky is darkened and glossed by the wetness, and nothing
under an overhang is. Puddles lie on flat ground in the low places of a broad noise, on
roads, bare earth, and banks; pasture and fields drink the rain and seldom hold any. A
puddle is written as water (`~ - =`, `water`, `H2O`) and rings in rain, as the river does.

**Rain.** Streaks in a square of 160 by 160 columns around each view, fixed to the
ground so they do not swim as the eye moves, each column one drop falling at 7.5 m/s,
slanted by the wind, never thinner than a pixel and fainter as it widens. Where it lands
on the terrain, a splash for a sixth of a second: up close a `'`, further off a fleck.
Down walls, rivulets in broken lengths of `|`. Rain is drawn after the water, blended,
testing depth itself. It falls everywhere for now: shelter comes with the buildings in
S04, as a map of what is overhead.

**The wind's sentences.** Six, in `assets/text/en/wind.ron`, taken word for word from the
canon's Ledger voice (the bible and the prologue) until the owner writes the Vale's own.

**The wind that reads.** At most one gust in each 150 seconds, at a time fixed by the
clock, while the wind is strong enough. It is laid where the first view's gaze meets
the grass, no nearer than 14 m, its letters as wide as the view allows and long away
from the eye as the eye looks across rather than down on them. Over 3.2 seconds it
writes the line from start to end, holds it, and lifts it away in the same order.
Inside its strokes the blades bow downwind and shine, and so does the ground's grass
beneath them. The mechanism is in place; making it legible is S02-T10b, with the owner.

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
