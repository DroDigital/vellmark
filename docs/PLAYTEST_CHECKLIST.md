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

## Standing items

Added as each system is built.

| What | Since | How to check |
|---|---|---|
| The performance targets in `docs/TECH.md` | S00 | `perf-run.command`, and send the report |
