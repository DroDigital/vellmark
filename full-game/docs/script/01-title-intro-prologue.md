# Script: title, intro, and prologue

## Format

```
CHARACTER: Spoken line.
  > Player option -> what follows
  [Skill 30] > Option requiring a skill level
[Direction in brackets]
TEXT: Words that appear on the world or screen.
FLAG: state to record
```

All game text must be printable ASCII. `WITNESS:` marks optional co-op lines used when a second player is present. NPCs never use the player's name or gender.

## Title sequence (30 seconds)

[Black. One enormous character fills the screen, so large that it reads as landscape. The camera pushes in. Its strokes are made of smaller characters. Those are made of smaller still. The ink thins to dawn sky. The letters part around a far valley, and the valley is real. Lantern smoke rises in small steps. From one row of smoke, the title assembles.]

TEXT: VELLMARK

TEXT: Press any button.

## Intro cutscene: "An Account of the Matter" (about 2 minutes 30)

[Landscapes only. Narration is typeset one line at a time, in the sky or across the ground. Low bowed strings, and the scratch of a pen.]

TEXT: In the beginning there was the Page.

TEXT: The Page was blank.

TEXT: This was considered untidy.

[Letters pour down a bare slope and become a river.]

TEXT: Six Authors took up pens.

TEXT: They wrote the rivers before they wrote thirst.

TEXT: They wrote the barley, and the tax on the barley.

[A long view of the Vale. Each field is made of tiny words.]

TEXT: Everything that is, is entered.

TEXT: Every stone. Every goat. Every child, on the day she is born, and the day she will not need to be remembered.

TEXT: To be entered is to be real.

TEXT: It is not always comfortable. But it is thorough.

[Dusk. A village, windows lit. A woman calls someone in for supper. We cannot hear the name.]

TEXT: Line 4,090. The College reports an increasing incidence of mild forgetting.

[The northern horizon. A thin white band.]

TEXT: Line 4,108. A valley.

[A valley, full of smoke and small lights. The letters lift off it, softly, in one long row, and it is white.]

TEXT: Line 4,108.

[Silence. The same view. Nothing there. An old shepherd walks past and does not look at it.]

TEXT: Nobody noticed.

[Darkness. A great page, lit by one candle. A hand holds a small, crooked knife. Writing fills the page.]

TEXT: ENTRY. THE CLERK OF THE LAST PAGE.

TEXT: NAME:

[The knife draws through the line. It is gone.]

TEXT: BORN:

[Gone.]

TEXT: OWES:

[Gone.]

TEXT: REMEMBERS:

[The knife pauses. It shakes slightly. Then it cuts.]

[White. Then, slowly, grass, seen from very close up. Each blade is a line of tiny letters. The view draws back until the letters are too small to see, and it is only grass. Wind.]

TEXT: ENTRY 4112.11.08.0001. Morning, Tithe Hill. Wind from the north. Nobody is here.

## Character creation

Occurs in-world during Q1, when Wick looks at you. Choices: people and appearance. Wick comments, and the dialogue never treats any choice as more or less human.

## Q1: Struck Through

**Location:** Tithe Hill, Scrivenhold Vale. **Teaches:** movement, reading, Emendation, combat. **Unlocks:** OPEN.

### Wick

[You wake in an open grave, on a heap of loose dirt. A man leans on a spade, looking down with professional interest.]

WICK: Now that is a poor way to start the day.

WICK: Are you dead? Take your time. I get paid either way, or I would, if the form had a box for you.

  > "Where am I?" -> WICK: In a hole I dug for a Mister Bell. Mister Bell is running late.
  > "Who are you?" -> WICK: Wick. Gravedigger. Sole proprietor. Who are you?
  > "I can't remember anything." -> WICK: Then you're ahead of most. Come on, up.

[The player climbs out. Character creation occurs as Wick squints at them.]

WICK: Hm. Let's have a look at you, then.

[On completion:]

WICK: Well, you look like somebody. That's a start. Name?

  > [The player tries to speak. The dialogue box prints a name, and it fades as it is typed.]

WICK: Say again?

  > [It fades again.]

WICK: No. See, that's a problem.

[He takes out a stub of pencil and a tally book. He writes. The marks slide off the page and fall as small grey letters into the grass. He watches them go.]

WICK: In forty years I've buried two bishops and a man who was mostly a horse. Never had the pencil refuse.

WICK: Tell you what. The stone on top of the hill writes down every blessed thing that happens up here. Go and see what it says about you. Mind the dog.

WITNESS: [If in co-op] WICK: And there's two of you. Lovely. I've only the one hole.

### The climb

[A short path. A Blot Hound Runt worries a bone beside the path. It growls, then notices you, and sits down wagging its tail. Dogs like you. From here it follows at a distance, and after the ossuary it can be befriended and named. Further on, a low stone wall is covered in small text. The prompt appears: Read.]

TEXT: HERE LIES BARTHOLOMEW RUSH. HE WAS OWED 11 MARKS. THEY KNOW WHO THEY ARE.

[At the top, the land opens. The whole Vale lies below: fields, the road, smoke, a walled city in the distance. To the north, a band of perfect white runs along the edge of the world. Hold on this view for a moment, with no prompts.]

[Far to the north, against the white, something black and level crosses the sky once. A watchtower on a distant ridge is ruled through and is not there. It is too far away to hear. Nothing remarks on it.]

[The Standing Page: a tall slab covered in letters. As you walk up, the letters resolve. They are moving. The newest line is writing itself as you watch. A crow lands on the stone.]

TEXT: ENTRY 4112.11.08.3391. A crow alights upon the Stone.

TEXT: ENTRY 4112.11.08.3392. Wind from the north.

[You step closer. Your shadow crosses the stone.]

TEXT: ENTRY 4112.11.08.3393. The grass bends beneath no one.

TEXT: ENTRY 4112.11.08.3394. Nobody is here.

[You reach out. A word on the stone lifts free. It is OPEN. It settles into your hand as ink.]

TEXT: You have learned a word. It is not yours. Nothing here is.

### The ossuary

Wick's tools are locked in the ossuary at the foot of the hill, and Mister Bell's grave cannot be finished without them. The ossuary's doors are two bronze leaves three times the player's height, shut for forty years. Their Inscription reads `LOCKED`. Replace it with `OPEN`: both leaves are thrown back with the full effect, and every lesser lock on the hill springs with them, gates, grave-lids, and bone bins alike. This is the player's first cast and must be the biggest thing they have seen so far.

Inside are a small crypt, a pressure puzzle, and two Shambling Entries, each muttering its own debts. Combat is taught here. A Bone Clerk at the end drops Wick's shovel.

SHAMBLING ENTRY: - owes for a pig - owes for a second pig -

### Return

WICK: Well?

  > "It says nobody's here." -> WICK: Does it. And here's you, holding my shovel.
  > "It wrote that the grass bent under no one." -> WICK: Grass has always been a gossip.
  > "What's the white in the north?" -> WICK: That's what we don't talk about. Keeps the rates down.

WICK: You want a clerk. Ambry, down the road, there's a fellow does nothing but small matters. If you can't be written down there, you can't be written down at all.

WICK: Stop off at Little Penance on your way. Mabel Oake's owed a headstone refund. Give her this, and have something to eat.

[He hands over 12 Marks in a twist of paper.]

WICK: And mind how you go. Whatever you are.

FLAG: q1_complete
FLAG: wick_met
