# Playtest checklist

What only the owner's Mac can prove. A cloud session proves that things are correct; it
never proves that they are fast, that they look right in motion, or that they feel good.
Each item says exactly what to do. Tick it, or write a sentence about what was wrong.

## How to get a build

1. Open the repository on GitHub, then **Actions**, then the newest **CI** run on `main`
   with a green tick.
2. At the bottom of the run's page, under **Artifacts**, download `vellmark-macos-N`.
3. Unzip it. Inside are `Vellmark.app`, `perf-run.command`, and `READ-ME-FIRST.txt`.
4. The first time, right-click `Vellmark.app`, choose **Open**, and choose **Open** again.

## Phase S00: foundation

There is no game yet. This checks that the build reaches the Mac and can be measured.

- [ ] `Vellmark.app` opens a window showing a sky and a ruled plain
- [ ] The mouse looks around and W, A, S, D move, smoothly
- [ ] F1 shows the profiler view, and its letters are sharp
- [ ] The grave-accent key opens the console; `hour 18` then Enter turns the light to dusk
- [ ] Escape frees the mouse, and Escape again closes the game
- [ ] Right-click `perf-run.command`, choose **Open**. A window flies for about half a
      minute and closes, and `vellmark-perf-report.txt` appears on the Desktop
- [ ] Send the report file back

## Phase S01: the Look Gate

Five minutes in the yard. It passes when you would put a screenshot of it on a store page.

Keys: W A S D and the mouse to move and look. Hold **R** to read. **F5** lights the lantern and puts it out. In the console (the grave-accent key), `hour 12`, `hour 17.7`, and `hour 23` set noon, dusk, and night, and `hour run` lets the clock go.

1. **Stand far.** You start four metres inside the gap in the south wall. Walk backwards about ten metres.
   - [ ] From across the yard it looks like a real place, and nothing gives the letters away
2. **Walk in.** Walk slowly up to the north wall, to the right of the door.
   - [ ] The letters arrive without a pop or a crawl
   - [ ] At about two metres you can read the wall's ledger around the door
   - [ ] At arm's length the small letters are sharp, and heavier where the larger letters' strokes pass
3. **Read the bench.** Crouch is not built: stand over the bench by the east wall and look down at its seat.
   - [ ] It can be read, and it is worth reading
4. **Hold Read.** Stand back in the middle of the yard and hold R. Let go. Do it at night as well.
   - [ ] Close reading made you say something out loud
5. **Dusk and the lantern.** `hour 17.7`, then `hour 23`. Press F5 twice.
   - [ ] Dusk and lantern light are beautiful
6. **Smoothness.** Press F1 for the profiler and walk around.
   - [ ] It is smooth, and the frame line stays under 16.7 ms
7. **The depth looks.** Stand within a metre of the north wall and press F6 to step through flat, atoms, lattice, and source. Move: strafe and lean.
   - [ ] One of them makes you feel you are looking into the wall. Which: ______
   - [ ] The wall's sentences can still be read in it
8. **Tune.** Press F4. Down picks a row, left and right change it.
   - [ ] Write down the reveal and contrast values you like best: reveal ____ contrast ____
9. **Measure.** Quit, then run `perf-run.command` and send back the report.

If something is wrong, a sentence is enough: where you stood and what you saw.

## Phase S03: the Play Gate

Fifteen minutes in the valley, with sound on. It passes when you want to keep playing
after the steps run out. Get the build from the newest run of pull request 7 (the
**macOS app for the owner** artifact), as above.

1. Start the game (`Vellmark.app`). You stand by the top of Tithe Hill. Walk to the tall
   slab ahead of you, hold **Read** on it, and keep reading until it gives you `OPEN`.
   Try both a controller and the mouse for the walk.
2. Read a few of its lines close: hold Read with nothing in view to read up close, and
   watch time slow (it costs Stamina).
3. Walk down to the bridge road and follow it west, to two fenced yards (about 57, -62;
   or console `goto 57 -62`). The south yard's gate is `LOCKED`: walk at it,
   then write `OPEN` on it (tap the off-hand button while looking at it).
4. Write `LIT` on the lantern by the gate (console `learn LIT`; its Page comes in S06).
5. Step into the yard. Fight the Inkling with the sword (tap, tap, tap; hold for a heavy
   blow). When it swells and rears, sidestep. Then fight it again (console `respawn`)
   writing `COLD` first (`learn COLD`): it stops splitting.
6. In the north yard, the three annotated ones: hit the `LOCKED` one's front, then write
   `OPEN` on it charged (hold the off hand). Get bitten by the `BURNING` and `COLD` ones.
7. Hold both hands' buttons together with `BURNING` in the off hand: the blade carries it.
8. Type a word's typo on the keyboard while it charges (`TURNING` for `BURNING`): it goes
   wrong, and you hear it go wrong.
9. Walk east along the road to the river. Read the Page at the bridge's head, and write
   `WHOLE` on the bridge, fully charged. Walk out onto it as it comes back. Then do it
   again (console `break mended_bridge`).
10. Pick up a crate by the road (Interact) and throw it (main hand) at the keg.

The owner checks:

- [ ] Moving and looking feel right at once, on a controller and with a mouse
- [ ] Reading a wall was worth doing
- [ ] The first word obeyed, and it felt like power
- [ ] The bridge coming back together made you do it twice
- [ ] Swinging with one hand and writing with the other felt like one way of fighting
- [ ] The sidestep was enough defence
- [ ] A typo or a compound happened by accident and made you laugh or try again
- [ ] Every word looks and sounds different, and expensive
- [ ] It stayed smooth through all of it: with the fight's every effect on screen, read the
      report's frame lines (`perf-run.command` after the fight, or F1 during it)

## Standing items

Added as each system is built.

| What | Since | How to check |
|---|---|---|
| The performance targets in `docs/TECH.md` | S00 | `perf-run.command`, and send the report |
| The opaque pass inside its 5.0 ms share | S01 | The `opaque` line of the report's `[gpu passes]` |
| No shimmer or swimming letters in motion | S01 | Walk and strafe along the north wall at 6 m, 2 m, and 0.5 m. The cloud's shimmer check passes; only a real screen shows motion |
| Grass and foliage inside 2.0 ms | S02-T07 | Run `valley.command`, stand on Tithe Hill, and read the `opaque` and `depth` lines with grass in view. The grass's density is 160 blades a square metre within 8 m; say if it should change |
| Grass that moves well | S02-T07 | In the valley, walk through the pasture and the barley: the blades part around you and spring back, and the wind travels across the field in gusts. With the free camera (F2, Left Ctrl to go down), bring your eye to a blade: it is a line of words, `grass C6H12O6` |
| Water inside 0.9 ms | S02-T09 | Stand on the river's north bank (about 0, -100) facing south and read the `water` line of the report |
| Water that looks like water | S02-T09 | The river and the mill pond at noon and at dusk: does it read as a northern river, dark and mirroring? Kneel at the edge (F2, Left Ctrl): the foam is `.` and `,`, and letters gather on the ripples |
| Rain that feels like rain | S02-T10a | In the valley, console `rain 1`. Watch the sky close, the road darken and puddle over a minute, splashes at your feet (`'` when you look close), and the river ringed. `rain 0`, and it dries over eight minutes |
| The wind that reads | S02-T10b | Stand at the foot of Tithe Hill looking up it, or look out over open pasture, and type `wind 0` to `wind 5`. Can you read the sentence as the gust passes? Say what would make it readable: bigger, nearer, stronger, slower |
| Rain inside its budget | S02-T10a | With `rain 1`, read the `rain` and `water` lines of the report |
| Shadows inside 2.0 ms, and no pop when the far cascade is redrawn | S02-T11 | In the valley, read the `shadows` lines of the report. Watch the far hills' shadows as the sun moves: they should step so little that you cannot see it |
| The governor and the presets | S02-T12 | `perf-run.command` runs on Medium. In its report, the `[governor]` part says what was given up and when; nothing given up is best. In play, the console's `preset high` and `preset low` should change grass density with no stall |
| Trees that look like trees | S02-T08c | Walk from the field into the east wood (about 160, 120, facing east) and through the Quiet Orchard (150, -178). Do the woods read as oak, ash, and hawthorn, and the orchard as apples? Watch a tree as you walk away from it: the change between levels should not be seen. Stand under an apple (F2 to fly up into it): its leaves are `^ A v`. Inside a crown, its twigs pass the eye as thin lines. Say what is off: shape, colour, density, the bark's seam up the trunk |
| The valley sounds like a place | S02-T13 | Listen to `docs/evidence/S02/ambience.wav` (a breezy morning with birds, then rain by the river), then walk the valley with sound on: from the pasture into the east wood (leaves in the wind), down to the river, `rain 1`, `hour 22` (the birds stop). Say if any layer is too loud, too quiet, or sounds made rather than heard |
| Bark that records its age | S02-T08d | Walk up to an oak in the east wood (about 227, 118) and an apple in the orchard (150, -178) and read the bark at arm's length: `Oak. Stood 130 years.` Walk on and back: the trees nearest you carry their records, with no stall when a record moves to another tree. Say if the wording or the size of the letters should change |
| Trees inside their budget | S02-T08c | On Tithe Hill and in the east wood, read the `opaque`, `depth`, and `shadows` lines of the report with trees in view |
| Grass seen low, and no shimmer in it | S02-T07b | Stand in the pasture east of the hill (about 160, 120) and look along the ground, at 10:00 and at dusk (`hour 17.8`), and walk slowly sideways. Nothing should crawl. Then compare the near grass with `docs/evidence/S02/grass_grazing.jpg`: the dark strokes between blades are softer than they were. Say if the field looks flatter than it should |
| Fog, and no shimmer anywhere | S02-T14 | In the valley, console `fog 0.5`, then `fog 0`. Walk slowly past the barley at noon, at dusk (`hour 17.8`), in rain, and in fog: nothing should crawl or sparkle |
| Dusk from the hill is a picture worth keeping | S02 exit gate | Stand on Tithe Hill at `hour 17.8` and look north over the barley. Yes or no |
| A word's magic that feels like writing | S03-T06 | In the yard, hold `BURNING` (Q) and press, then hold, the off hand at the door: the charge at the hand, the word's flight as a light, the impact, the wave over the door, the embers after. Say if any part is too faint, too long, or too much |
| The bridge coming back together | S03-T08 | In the valley, walk the bridge road to the river (about 130, -100), hold `WHOLE`, charge, and write it on the bridge. Its letters rise from the Quill and it knits outward from you; walk out on it as it comes. Is it exact enough to be the signature of destruction? Say if it is too fast, too slow, or pops |
| Props with weight | S03-T08 | By the bridge road (about 125, -93), look at a crate and press Interact: it is held before you. Walk, turn, put it down, pick it up, and throw it (main hand) at the bank or the keg. Does it feel heavy enough? Does a hard throw break it into letters? |
| Physics inside 2 ms | S03-T08 | With the crates thrown and letters falling, read the `sim` line of the report |
| Swinging that is simple and weighty | S03-T09 | Until the Inkling comes (S03-T12) there is nothing to strike but the air: tap for the chain, hold for the heavy attack, hold the off hand to block with it empty. Does the timing feel right? Is the hit stop felt or missed? |
| Words on the blade | S03-T10 | Hold `BURNING` in the off hand and press both hands together: the blade carries it. Do the same with `OPEN`: the refusal. Does writing on your own weapon feel like a trick worth doing in a fight? |
| The bow | S03-T10 | (Once the bow is in the hands, S03-T11.) Draw and loose at a mark across the river: does the arrow's drop read, and does a full draw feel worth its Stamina? |
| Hands that feel like yours | S03-T11 | Walk, swing, hold for a heavy blow, block with a shield, draw the bow, and charge a word. Do the hands sit where you expect? Is any pose too close, too low, or too stiff? Does the charge read as writing? Compare `docs/evidence/S03/hands.jpg` |
| The Inkling fight | S03-T12 | Walk to the yards off the bridge road's west end (about 57, -62). Fight the plain Inkling with the sword alone, then with words: does its gathering give you time to sidestep? Does splitting feel like a problem worth `COLD`? Then the LOCKED one: strike its front, then write `OPEN` charged. Is it readable as ink, up close and at range? |
| Every word sounds different, and expensive | S03-T13 | Listen to `docs/evidence/S03/motifs.wav` (each word, then its typo going wrong, then the fight's sounds), then in play write each word and a typo. Does each word sound like itself? Does the wrong note make you laugh or wince? Is any combat sound thin or too loud? |
| The Standing Pages | S03-T14 | Walk up to the Tithe Hill Stone from the start and hold Read on it: does taking `OPEN` feel like a moment? Then read the Page at the bridge's head: does its text (new, in the Ledger's voice) belong? Say if it should change |
| How long a fight lasts | S03-T14 | The scripted bot wins the yard's fight in 3 to 7 s because it sidesteps every lunge. Fight it yourself with each build: too short, too long, or right? |
| The pack | S05-T07a | Take a few drops, then press Tab (Back on a pad). Move with W/S or the stick, equip with E, close with Tab or Escape. Is it quick to read? Is any name or line hard to take in? Compare `docs/evidence/S05/inventory.jpg`. Read the report's frame times with it open |
| The trade screen | S05-T09 | Open the console (`) and type `trade`. Buy with E, move to your pack with D or the stick, sell, try to buy what you cannot pay for, leave with Escape. Are the prices and the refusals plain at a glance? Compare `docs/evidence/S05/trade.jpg` |
| The pause menu and settings | S07-T05 | Press Escape (Start on a pad). Walk every page with the arrows and Enter, then with the pad. Move a button (Controls, Buttons), turn the quality to High and back, set the interface size, and quit and relaunch: is everything as you left it? Delete `settings.ron` and launch: which preset did it pick on the Mac? |
| Captions, states, contrast, letters | S07-T04 | In Accessibility, turn on Say creatures' states and High contrast, set Letter size to the top, then fight the Inkling with words. Are the captions (bottom of the screen) quick enough to follow? Is anything cut off at the larger letters? Compare `docs/evidence/S07/access_high.jpg` |
| Reduced motion and fewer flashes | S07-T04 | In Accessibility, turn on each in turn and fight the Inkling with sword and `BURNING`: does each do what it says, and nothing else? |
| The murmur, and voices | S06-T15 | Listen to `docs/evidence/S06/murmur.wav`, then `talk wick` in play. Does the murmur sit under the text without tiring? Does Wick sound like an old, unimpressed man? Then choose: voice the opening and the supper (one batch of credits, with your approval), or keep the murmur |
| Footsteps | S04-T15a | Walk from the pasture onto the road, through the barley, down the bank into the river, and over the mended bridge, then sneak. Does each ground sound like itself? Is the pace of steps right at a walk and a sprint? |
| Obelus | S04-T14 | On the bridge road (`goto 112 -86`), open the console and type `obelus`: it comes down the view from behind and strikes the crates by the road. Does it frighten? Does what it struck read as crossed out? Then on Tithe Hill, `obelus high`: does it read, far off, as something black and level crossing the sky once? Compare `docs/evidence/S04/obelus.jpg` |
| Photo mode | S07-T03a | Press P on Tithe Hill (or Escape, Photo mode). Fly about with WASD, Space, and Ctrl; set the view to Split and turn and move the divider across the Stone; save a picture and find it in `Pictures/Vellmark`. Is the split frame the picture you would share? Is the panel easy on a pad? Compare `docs/evidence/S07/photo.jpg` |
| The Stone writing | S06-T10b | Walk up Tithe Hill to the Stone slowly and watch its face: from about 20 m the newest line writes itself; step close and it writes that the grass bends beneath no one, then that nobody is here. Is it legible as it happens? Is 11 letters a second right? Compare `docs/evidence/S06/page_writing.jpg` |
| A word given | S06-T10a | Read the Page at the bridge's head (about 128, -97) until it gives `WHOLE`: does the word lifting and holding feel like a moment, and the bridge mending by itself like a reward? Is 5 s held too long? Compare `docs/evidence/S06/spectacle.jpg` |
| The title | S06-T13a | Run `valley.command` and watch the title through without pressing anything (30 s): does the dive feel like falling into letters? Does the valley arriving through the parted letters land? Then launch again and press a key early: the title should come at once. Turn on reduced motion and launch: a plain fade. Compare `docs/evidence/S06/title.jpg` |
| The score | S06-T14 | Listen to `docs/evidence/S06/score.wav` (a day's phrase, a night's, then a fight), then play: stand in the valley for two minutes by day, `hour 22` for two more, then fight the Inkling. Is the theme memorable? Is the rest between phrases too long or too short? Is the fight's music too loud against the blows? Say what the synthesized instruments most need from real samples |
| Talking to Wick | S06-T06 | Open the console, type `talk wick`, close it, and press E on him. Go through the grave: does the line write itself at a good speed? Does a press finish it, and the next go on? Are the choices easy to take with the mouse hand on the keyboard, and with a pad? Then `talk wick` again: he has nothing more until the shovel is found |
| Save and continue | S05-T11 | Play a while (break the stool-sized things, take a drop, mend the bridge), quit from the pause menu (Escape, Quit to desktop), then start with `--continue` (or console `load`). Is everything where you left it? |
