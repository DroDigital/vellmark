# Script: Act Three, "Colophon"

## Q15: Whoever Remains

**Location:** Open world. **Purpose:** gather support for the road north.

[The player has learned the truth. Marshal Quillon or Steward Ellery calls a meeting in the bread queue, as it is the only place left where everyone is reliably present.]

ELLERY: I have 19 days of flour, and a white line I can see from the granary. I'd like someone to tell me what we are doing.

The player visits any of the allies below. Each completed questline adds a real effect in Q16 and Q19. Nothing here is required.

| Ally | Requirement | Effect |
|---|---|---|
| Gudrun Anvil's army | Q10 complete, or Anvil's trust | Holds the Vale line, delaying Stage 3 visuals and saving named settlements in the epilogue |
| The Errata | Errata questline | Clears Scrapers from the Unwritten Road |
| The College | College questline | Provides rubric-ink stores, reducing Ink costs in the Gutter |
| Margin Folk | Margin questline | Opens a secret route through the Scriptorium |
| Ledger Guard | Guard questline | Fortifies the Gutter's entrance |
| Blank Choir | Choir questline | Gentle passage: Erasure effects are weaker on you |
| Sorrel's name-keepers | Q5 and Sorrel's trust | Prevents losing map labels on the road |

Each ally receives a short scene. Sample:

HOB: How much?

  > "It's the end of the world." -> HOB: I've seen the end of the world. Twice. It was invoiced in thirds.

ELLERY: Is there a form for this?

  > "I don't think there is." -> ELLERY: Then I'll make one. Someone will want it after.

## Q16: The Unwritten Road

**Location:** The Erasure. **Boss:** Deleatur. **Reward:** NAMELESS.

[Past the last signpost, the world stops being described. Fences are faint. Words on the HUD lose letters: "COMPASS" becomes "COM ASS" becomes "C  A". Music thins to a single held note.]

### Last Ditto

[A cluster of half-houses. Ilse, the headwoman, sits with a small group, saying names. She has 19 words left.]

ILSE: You. Good. You. Sit.

ILSE: We. Say. Name. Name. Name.

  > "Whose names?" -> ILSE: - [She shakes her head. She touches her chest.] Gone. Say. Anyway.

[The player may add a name to the recitation. Options include Tamsin, Odile, and every other erased person the player has encountered. Each is added and repeated by the group. It is pointless in any practical way, and the game should give it full time.]

ILSE: Good. You. Good.

Deleatur attacks on the road beyond. It does not fly in lines. It loops, the printer's mark for removal, and its loops erase the ground. The fight takes place on dwindling islands of described terrain. Players can use WHOLE to re-describe ground.

TEXT: NAMELESS.

[The ground Deleatur looped away writes itself back as a plain road running north. Nothing else returns.]

## Q17: The Last Scriptorium

**Location:** The Last Scriptorium. **Reveals:** the Authors, and the Corrector's offer. **Triggers Erasure Stage 3.**

[A plain stone building on a white plain. Desks in rows. The candles are still lit. On a lectern in the central hall, as tall as a house, lies the Great Ledger, open at its last leaf. A pen rests on the page, waiting.]

[Text appears on the Ledger.]

TEXT: NOTICE. The Clerk is reminded of an outstanding instruction.

  > "Who are you?" ->

TEXT: This office is the Corrector. It reviews entries for accuracy.

  > "You erased a child." ->

TEXT: An entry was found to be surplus. No action is required of you.

  > "Why?" ->

[A long pause. The text does not appear for several seconds.]

TEXT: This office has not previously been asked.

TEXT: There is no more room. Each new life requires an old one. This has been so for 1,100 Lines. The remaining supply of old lives is exhausted.

TEXT: Entries 1 through 6 cannot be removed. Their combined annotation occupies forty-one percent of the Ledger.

  > "Entries 1 through 6?" ->

[The first page of the Ledger turns itself. Six names, in six hands, in ordinary, slightly nervous handwriting. Each is followed by a long, crowded column of later additions.]

TEXT: ENTRY 1. BERTRAM ASKEW, CLERK. Amended: THE RUBRICATOR, JUDGE OF ALL.

TEXT: ENTRY 2. MAUDE VELLAM, CLERK. Amended: MOTHER VELLUM, FROM WHOM ALL FLESH.

TEXT: ENTRY 6. TOM SLIPPER, APPRENTICE. Amended: SAINT SLIP. Amended: Sorry. Amended: I only wanted to see if it would work.

  > "They were people." -> TEXT: They were the first to hold the pen. This office has no opinion.
  > "Then scrape them." -> TEXT: Entries 1 through 6 are Sealed by their authors. That was the first thing they wrote.

TEXT: Three outcomes are procedurally available.

TEXT: REBIND. A Remainder is selected. It is scraped. The Ledger is resewn, and will suffice for approximately 900 Lines. The Clerk's status will be regularized. The Clerk will not recall this.

TEXT: FINAL LINE. The Clerk writes the outstanding instruction. All entries are scraped. The work begins again. No one will suffer. No one will be.

TEXT: A third outcome is not available, but this office notes that the Ledger is made of skin and thread.

[At this point, Thale's forces arrive, and the Cantor's people, and a Margin Folk runner. The three of them are already in the hall when the player looks up. There is no fight, not yet. There is a very tense conversation.]

THALE: You have read it. So you see. It holds for nine hundred years.

YSOLDE: Nine hundred more years of mothers reading birthdays.

INTER PELL: Or. And I want everyone to stay calm. We could steal it.

[The player chooses which route to follow. This choice can be changed up until the end of Q19.]

FLAG: authors_truth_known
FLAG: erasure_stage_3

## Q18A: The Remainder (Restoration route)

**Ally:** Thale. **Goal:** gather binding thread and agree on the list.

Thale takes the player to the Scriptorium's bindery, along with Marshal Quillon if she remained loyal. Binding thread is stored in six locked presses, each one guarded by a past clerk. The Remainder list must be completed, and the player may reorder its first page.

THALE: You may move any name you like. You may not remove a total. That is the only kindness arithmetic allows.

  > [Player may move the Blotmere, Saltmargin, the Vale's Gutters, or a Quire of their choice. This decides what the epilogue says is lost.]

THALE: Thank you. I hated doing that alone.

## Q18B: The Long Rest (Blank route)

**Ally:** Ysolde. **Goal:** say goodbye, and prepare the hand.

Ysolde asks the player to visit five people, and to tell each whatever they think is kindest. There is no combat in this quest, until Ser Anselm or Thale's troops arrive at the Scriptorium.

If the player has found Lise's impression, Ysolde can be told.

  > "Lise is in the Verso. She wants to be older." ->

YSOLDE: [After a long silence.] Is she in pain?

  > "No."

YSOLDE: Then let me keep walking, before I stop.

Sample visit, Mabel Oake, if the player went back:

MABEL: You look done in. Do I know you? Sit. I've laid extra.

## Q18C: The Greatest Forgery (Free Hand route)

**Ally:** Inter Pell and the College. **Goal:** forge an instruction that closes the Ledger.

A heist on the most heavily guarded document in existence. The team needs the following: a sample of the Corrector's form language (College), thread cutters (Tace), six Author signatures (forgers and the Author quest artefacts make it easier), and a distraction big enough for both Thale and the Cantor.

INTER: Nobody reads the small print. Not even God. That's the whole business model.

SABLE: I want it noted that this is the first time I have ever agreed to falsify a citation.

## Q19: The Gutter

**Location:** The seam between the world's pages. **Final dungeon, shared by all routes.**

[Beneath the Ledger, the player descends into the binding of the world: a colossal canyon of stitched vellum, with one wall showing the Recto in colour and the other the Verso in white on black. Thread as thick as trees runs between them.]

The dungeon's rooms replay what the player has done. Rooms contain impressions of erased places and people, depending on flags.

- Little Penance, as it was, for one lane's length
- The Stacks, with Odile's handwriting on the wall
- A room with any ally who died
- A long corridor in which the Long Queue statues turn to watch you

TEXT: Do try to stand up straight.

Allies from Q15 give their effects here. The player may change routes at the door to the final chamber.

## Q20: The Last Line

**Location:** The Last Leaf. The final boss depends on the route. See `05-endings.md`.

[A room with no walls. The Ledger, the pen, and the last, nearly full leaf. There is room for exactly one line. The player's rivals stand or lie as the route has left them.]

TEXT: The Clerk is invited to complete the entry.

In co-op, both players must agree. If they don't, the Ledger waits. It has all the time left that there is.

WITNESS: TEXT: A witness is present. This is in order.
