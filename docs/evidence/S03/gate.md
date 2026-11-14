# S03 exit gate

Run on 2026-11-14 on `claude/charming-bardeen-5aqf5c`, software rendering (llvmpipe).

| The gate asks | Result | Evidence |
|---|---|---|
| The scripted run passes | Passes: read, learn, open, light, mend, and fight | `crates/harness/tests/scripted_run.rs`, `scripted_run.md` |
| The word matrix has a result or a refusal for every word on every kind of target that exists | Passes: the bake refuses a sheet without an outcome for every one of the 13 target classes, for the word and its typo, and every class a scene uses is among them | `docs/WORDS.md` (182 rows), `crates/bake/src/words.rs` |
| All three builds win the fight | Passes: blade and word, words alone, steel alone | `scripted_run.md` |
| The frame budget holds on the owner's Mac during a fight with every effect on screen | Waits on the owner | `docs/PLAYTEST_CHECKLIST.md`, Phase S03 |

The full suite passes: 54 test groups, `cargo test --workspace`.

The phase's evidence: `reading.jpg`, `open.jpg`, `magic.jpg`, `mending.jpg`, `hands.jpg`,
`inklings.jpg`, `motifs.wav`, `scripted_run.md`.

The owner gate, the Play Gate, is in `docs/PLAYTEST_CHECKLIST.md` under Phase S03.
