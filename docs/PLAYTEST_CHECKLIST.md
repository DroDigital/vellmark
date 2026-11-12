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
| The governor and the presets | S02-T12 | Run `perf-run.command` once with each preset (`--preset low`, `medium`, `high`). Send the three reports: the `[governor]` part says what was given up and when. In play, the console's `preset high` and `preset low` should change grass density with no stall |
