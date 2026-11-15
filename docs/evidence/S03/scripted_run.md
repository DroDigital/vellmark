# S03 scripted run

`crates/harness/tests/scripted_run.rs`, in the real valley, through `World::step` alone,
on 2026-11-14:

1. **Read and learn.** The bot reads the Tithe Hill Stone (its text is the prologue's) and
   takes `OPEN`, which settles into the writing hand.
2. **Open.** The south Inkling yard's gate is `LOCKED` and stops a body walking at it.
   `OPEN` written on it, its Inscription says `OPEN` and the bot walks through.
3. **Light.** `LIT` on the lantern by the gate lights it. (`LIT`'s Standing Page, the Lamp
   That Was Promised, is built in S06; the run gives the word.)
4. **Mend.** The bot reads the Page at the head of the Mended Bridge and takes `WHOLE`,
   writes it charged on the bridge, and the bridge stands whole; it walks the length of it
   to the far bank and never drops toward the river.
5. **Fight.** The plain Inkling in the south yard, with each build:

```
BladeAndWord: won in 4.8 s with 100 health left
WordsAlone: won in 7.3 s with 100 health left
SteelAlone: won in 3.2 s with 100 health left
```

The bot sees each lunge gathered and sidesteps it, which a person will not always do;
how long the fight should last is for the owner's playtest (`docs/PLAYTEST_CHECKLIST.md`).
