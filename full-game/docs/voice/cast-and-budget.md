# Voice cast and budget

The written game stays complete. Voice is used only where it does the most: every key story scene, plus a small set of follower, faction, local, and crowd lines. Everything else is typeset text with a synthesized murmur shaped by the speaker's preset.

`voice_lines.tsv` is the recording script. It holds 263 lines and 13,705 characters.

## Rules

- **A scene is either fully voiced or not voiced.** Never add an unvoiced line inside a voiced scene. Connective dialogue elsewhere is text and murmur.
- **The Ledger never speaks.** The Corrector, inscriptions, the intro, and outro narration stay silent and typeset, with pen sounds. Voice belongs to people.
- **The player and the Witness are silent.**
- Do not generate a line until its text is locked. Every change costs credits.
- All generation goes through `tools/generate_voice.py`, which enforces the limit.

## Credit budget

| Use | Credits |
|---|---|
| P1: story scenes, 180 lines | 10,457 |
| P2: followers, factions, local and side moments, 50 lines | 2,682 |
| P3: crowd lines, 33 lines | 566 |
| **First takes** | **13,705** |
| Retakes, up to the tool's limit | 22,295 |
| **Tool limit** | **36,000** |
| Creating the 10 voices and testing them | 4,000 |
| **Account total** | **40,000** |

This assumes 1 credit per character. Generate one line first and check the recorded cost. Even at 2 credits per character, one full set of takes would fit.

## The 10 custom voices

All are adults and synthetic. None imitates a real person. Build each with Voice Design, then confirm it works with Eleven v4 before using a slot for the next. Test line for all: "I have a key, and nothing it opens. Sit down. It's only a little further."

| ID | Name | Voice Design description |
|---|---|---|
| B1 | The Reader | Elderly woman, thin dry voice, precise consonants, unhurried, faintly amused, very close to the microphone |
| B2 | The Hearth | Warm middle-aged woman, soft rounded tone, gentle country accent, slightly tired, kind |
| B3 | The Stillness | Low, calm woman, smooth and quiet, long even phrasing, almost no vocal effort |
| B4 | The Spark | Light, bright young adult woman, clear high placement, quick and curious, expressive |
| B5 | The Gravel | Old man, gravelly and weathered, slow rural speech, wry and unimpressed |
| B6 | The Minister | Refined grey-voiced older man, clear baritone, exact and courteous, permanently tired |
| B7 | The Bell | Strong adult woman, big chest voice, commanding alto, direct, carries across a yard |
| B8 | The Nib | Thin, nasal tenor man, fussy and quick, anxious and over-articulated |
| B9 | The Ox | Very deep large man, heavy and slow bass, few words, rough edges |
| B10 | The Fair Copy | Neutral adult voice, androgynous, perfectly even and clean, no regional accent |

## Character presets

Each preset is applied by the game's audio pipeline. Pitch is in semitones and does not move the formants. Formant shifts vocal size separately. Rate is playback pacing.

| Preset | Base | Pitch | Formant | Rate | Treatment |
|---|---|---|---|---|---|
| ODILE | B1 | 0 | 0% | 100% | Clean and intimate. The reference voice |
| ILSE | B1 | -1 | +3% | 72% | Thin, airy, with extended gaps between words |
| MIMI | B1 | +1 | -5% | 90% | Nasal mid boost, a hint of creak |
| MABEL | B2 | 0 | 0% | 100% | Warm and untreated |
| TACE | B2 | -3 | -8% | 108% | Forge-rough saturation, brighter upper mids |
| NELL | B2 | +2 | +2% | 116% | Quick and bright, slight smile lift at 2 kHz |
| ANNE | B2 | +1 | +3% | 88% | Soft, breathy, high frequencies gently rolled off |
| DROWNED PARISHIONER | B2 | -7 | -14% | 70% | Waterlogged low-pass, slow chorus of three detuned copies |
| YSOLDE | B3 | 0 | 0% | 96% | Dry and close, with no room sound on any line |
| RUE | B3 | +2 | +5% | 84% | Glassy presence, exaggerated gracious pacing |
| TAMSIN | B4 | +5 | +18% | 104% | Small-body voice, with light high-shelf boost |
| TAMSIN_IMPRESSION | B4 | +5 | +18% | 92% | As above, with reversed reverb tail and gentle high-pass |
| TAMSIN_REVERSED | B4 | +5 | +18% | 100% | Audio played backward |
| SORREL | B4 | -2 | -3% | 80% | Warm, patient, soft woody low mids |
| ELLERY | B4 | -1 | 0% | 118% | Clipped, efficient, small upper-mid edge |
| LULU | B4 | -3 | -6% | 88% | Sly and slightly husky, soft air layer |
| INTER | B4 | -4 | -9% | 112% | Smoky saturation, close and conspiratorial |
| WICK | B5 | 0 | 0% | 100% | Untreated, slightly dusty low end |
| PETER | B5 | -2 | -5% | 82% | Flat and salt-dry, hard high-frequency roll-off |
| STIG | B5 | +2 | +6% | 76% | Frail, cold, faint tremble at 5 Hz |
| ASHBY | B5 | +3 | +8% | 90% | Gentle, slightly reverent, soft stone room |
| JORY | B5 | +5 | +10% | 120% | Bored, fast, thin |
| THALE | B6 | 0 | 0% | 96% | Dry and exact, with tight consonants |
| VEXLEY | B6 | +3 | +7% | 112% | Oily, bright, slight sibilance boost |
| QUILLON | B7 | 0 | 0% | 104% | Clear and untreated |
| ANVIL | B7 | -2 | -7% | 106% | Loud, saturated, with chest boost |
| BRYN | B7 | -3 | -2% | 84% | Flat, cold, hollow midrange |
| NAN | B7 | +3 | +6% | 112% | Cheerful and rough, bright coastal edge |
| EDMUND | B8 | 0 | 0% | 112% | Thin and fussy |
| SABLE | B8 | -4 | -8% | 90% | Pompous, deeper, rounded lecture-hall room |
| SOUR | B8 | -2 | +4% | 96% | Sharp, cold, high-mid bite |
| QUENTIN | B8 | -5 | -10% | 92% | Relaxed, muffled |
| SAINT SLIP | B8 | +4 | +9% | 124% | Small, frantic, slight flutter |
| LULL | B8 | -1 | +2% | 76% | Soft and slippery, slight swampy low-pass |
| HASK | B9 | 0 | 0% | 94% | Untreated bass |
| HOB | B9 | +3 | +3% | 108% | Tired, businesslike, dry |
| ANSELM | B9 | -1 | +2% | 90% | Flattened pitch contour, no emotion or vibrato |
| GERRIT | B9 | +5 | +9% | 100% | Mild and ordinary |
| SHAMBLING ENTRY | B9 | -6 | -12% | 72% | Dry rattle, papery noise layer |
| SECOND EDITION | B10 | 0 | 0% | 100% | Spotlessly clean, doubled slightly too perfectly |
| CLERK | B10 | 0 | 0% | 98% | Same voice, damaged: added breath and small pitch instability |
| OFFSET MAN | B10 | +2 | +5% | 110% | Comic brightness, reversed room tail |
| INDEXER | B10 | -3 | -5% | 100% | Mechanical comb filter, drawer clicks |

### Crowd presets

Lines tagged `_CROWD` are shared across unnamed and minor NPCs.

| Preset | Used for |
|---|---|
| HASK_CROWD, QUILLON_CROWD | Guards |
| NELL_CROWD, EDMUND_CROWD | Merchants and clerks |
| ODILE_CROWD, WICK_CROWD, SORREL_CROWD | Villagers by age |
| YSOLDE_CROWD | Choir members |

Each NPC derives a fixed offset from its ID: pitch within 2 semitones, formant within 6%, and rate within 8%. The crowd offset must keep minor NPCs away from the presets of the named characters.

## Audio pipeline

Processing is built into Vellmark's own `bake` and `audio` code, and the player only ever hears processed voices.

1. Decode, trim, and normalize each chosen take.
2. Shift pitch while preserving the spectral envelope, then shift the formants separately.
3. Change the pacing without changing pitch.
4. Apply the character's EQ, saturation, breath or noise layer, modulation, and special effects.
5. Limit, and write the result to the pack files.
6. At runtime, add environment reverb, 3D placement, and ducking. YSOLDE ignores room reverb.

Each preset's character is defined in data, and its murmur uses the same pitch, formant, and pacing values.

## Checks

- **Scene check:** no two characters in one scene may share a base voice. The current list has no collisions.
- **Distance check:** measure pitch, formants, and spectral features. Fail any pair of named presets that are too close.
- **Listening check:** add a blind A/B test of presets to `docs/PLAYTEST_CHECKLIST.md`. Automated measures cannot prove that a voice sounds good.
- **Artifact limit:** if a shift sounds synthetic, reduce the shift and rely more on pacing and EQ.
