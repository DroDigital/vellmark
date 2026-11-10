# Script: Act One, "Recto"

## Q2: The Clerk of Small Matters

**Location:** Little Penance, then Ambry. **Reward:** Tamsin's Stone, 50 Marks.

### Little Penance

[A village of twenty houses, a pump, and a mill. It should feel like a place you would want to live. Lanterns are being filled. Mabel and Gerrit Oake are about to leave for market in Ambry.]

MABEL: A refund. From Wick. I'll have this framed.

MABEL: You'll eat something. It isn't a question. Tamsin! Lay another place!

[Inside. Three adults' places and a child's. Tamsin Oake, nine, sets down a fifth with enormous care, and then stares.]

TAMSIN: You don't have any writing on you.

  > "Should I?" -> TAMSIN: Everybody does. On your sleeves and that. It says who you are, if you get close.
  > "What does yours say?" -> TAMSIN: You're not allowed to read people without asking. But you can if you ask.
  > "I'm eating." -> TAMSIN: You can eat and look at stones. I do.

[If the player asks to read her sleeve:]

TEXT: TAMSIN OAKE. BORN 4103. KNOWS WHERE THE GOOD STONES ARE.

[She brings out a wooden tray of pebbles, each with a paper label in careful handwriting.]

TAMSIN: This one is Grey. This one is Also Grey. This one I found on a Tuesday, so that's its name.

TAMSIN: This one's for you. It hasn't got a label because I don't know what to put.

[She gives you a small speckled stone. Its item description reads: "A stone. Label: blank."]

TAMSIN: You can tell me when you find out.

FLAG: tamsin_stone_received

### The Line

[On the road to Ambry, in open country. The birds stop. Obelus comes down the road from behind, low, drawing itself along it. Everything it passes is struck through with a thick black rule: a milestone, a hedge, an empty toll hut. The line is coming up the road faster than a walk.]

[The player must get clear of the road. There is a ditch, a wall, and a gate that is `LOCKED`. Whatever the line crosses stands afterwards as black strokes with nothing behind them. Being caught by it wounds badly and does not kill on Standard.]

[Obelus does not turn. It goes on north and is gone. The struck things stay struck, and are marked so on the map. They write themselves back with the rest when Obelus falls in Q7.]

[No one is on the road, and no line of dialogue is added. If asked later, people in Ambry have always known the road to have a gap in it.]

### Ambry

[A market town. Edmund Tally's office is a narrow room with a very wide desk. There is a queue of one.]

EDMUND: Clerk of Small Matters. State your matter, and indicate its smallness.

  > "I need to be written down." -> EDMUND: A registration! Wonderful. Name?

[The name fades again. He dips his pen. He writes. The ink beads up on the page and rolls off onto his shoe.]

EDMUND: Hm.

[He takes a different pen.]

EDMUND: Hm.

[He takes a different pen, different ink, and a stone tablet with a chisel. The chisel skids.]

EDMUND: There appears to be an irregularity. I'm going to need you to fill out Form Nine, Declaration of Irregularity.

[The form cannot be filled out.]

EDMUND: Then I'll need a Form Twelve for the failure of the Form Nine.

[An old woman has been sitting on the bench by the door, with a cane across her knees and bandaged eyes. She has been reading the bench with her fingers.]

ODILE: Edmund, you'll hurt yourself.

EDMUND: Reader Marsh, this person does not appear to be permissible.

ODILE: Few of the best ones are. Come here, you. Give me your hand.

[She reads the back of your hand with her fingers. She reads it for slightly too long.]

ODILE: Well. Isn't that a nice clean page.

  > "You know what's wrong with me?" -> ODILE: I know what's wrong with most people. You'd need to be more specific.
  > "Can you read me?" -> ODILE: There's nothing to read. That's what is so interesting.
  > "Who are you?" -> ODILE: Odile Marsh. I check things. Sit down.

ODILE: Have you done anything peculiar lately? Moved a word, perhaps?

  > "I changed a door." -> ODILE: With your hands, off a Standing Page, on your first day. The College has people who've spent thirty years on a single candle. You mustn't tell them. It would be very funny.
  > "No." -> ODILE: You're a poor liar. Good. It's a skill that can be taught, and I'd hate you to have it already.

ODILE: I'm going to Scrivenhold. You should come, when you've done whatever it is you think you must do first.

[Across the square, market traders begin to look north. A bell starts up. It stops.]

## Q3: Little Penance

**Location:** Road to Little Penance. **Boss seen:** Obelus. **Tone:** the most important scene in Act One.

[Mabel and Gerrit are at their market stall in Ambry. The sky to the north goes wrong. Above the road, something flies that is the wrong kind of black: a shape with no surface, like a hole cut in the page. It moves in a long, straight, level line.]

[The player runs toward Little Penance. Ahead, houses stand in the evening light. Obelus passes above the village, once, from left to right, like a ruler being drawn across a sum.]

[The village's letters lift from the ground in long rows. Houses thin out to outlines, then to pale marks, then to nothing. Lanterns stay lit for a moment with nothing holding them, and then those go too. There is no sound at all.]

[The player reaches a wide, white, empty field. The road goes in on one side and comes out on the other. In the inventory, the stone's description changes.]

TEXT: A stone.

[The label field remains. Nothing else.]

[Back at the road, a carter comes along whistling and drives straight across the white as if it were nothing unusual.]

QUENTIN DRAY: Evening. Long dull stretch, this. Always has been.

  > "There was a village here." -> QUENTIN: Here? Nah. Never been anything here but road.
  > "Did you see that thing in the sky?" -> QUENTIN: Bit of weather.

### The Oakes

[At Oake Farm, on the far side of the white. Mabel and Gerrit are home, unpacking from market. Mabel is setting the table.]

MABEL: Oh, hello. You look done in. Do I know you?

  > "Where's Tamsin?" -> MABEL: Who, love?

  > "Your daughter. Tamsin." ->

MABEL: We've no children, love. It's just us and Gerrit's mother.

GERRIT: Never had the luck.

[The player can show the stone. Mabel holds it and frowns.]

MABEL: That's a nice stone.

[She gives it back. She finishes setting the table. Three people live in this house. She lays four places.]

[She looks at the table for a moment.]

MABEL: I always do four. Can't think why.

[She takes one away. A moment later, without noticing, she puts it back.]

[Do not play music in this scene. End the quest when the player leaves, or after the player has stood there long enough.]

FLAG: little_penance_erased
FLAG: player_remembers_tamsin

### Odile

[Odile is waiting on the road, sitting on her folded coat.]

ODILE: Well? What did you see?

  > "There was a village. A girl. They've forgotten her." ->

ODILE: Yes.

  > "You know about this?" -> ODILE: I know what it looks like when someone comes back from a place, carrying nothing, and doesn't know why they're crying.

ODILE: When it takes something, it takes the thing and everyone's memory of the thing. I can tell you it happens, because I've worked it out from the holes. I couldn't tell you the name of one thing it has taken. Nobody can.

ODILE: Except that you just did.

ODILE: Tell me about her. Everything. Slowly. Someone should write it down, and it can't be you.

[She produces a sheet and a pen, and writes, by touch, as the player answers.]

  > "She collected stones." 
  > "She named one Tuesday."
  > "She gave me this."

[She writes each one. As she finishes each line, the ink fades from the page. She keeps on writing.]

ODILE: Doesn't matter. It's the habit that counts.

## Q4: A Reader's Pass

**Location:** Scrivenhold. **Boss:** The Indexer. **Unlocks:** capital services and all faction introductions.

### The gate

JORY STAMP: Name and purpose.

[He can't record the name. He looks at Odile, and then at the queue behind you.]

JORY: I'll put "see attached."

ODILE: Nothing's attached.

JORY: That's correct.

### The Hall of Record

[Lord Registrar Corvin Thale meets every new arrival Odile brings, because he has learned to worry about what Odile brings. He is grey, courteous, and tired. Papers surround him in exact stacks.]

THALE: Reader Marsh. And an irregularity. Sit, please. I'm told the ink would not take.

  > "I'd like to know why." -> THALE: As would I. We have that in common. Let us keep it that way.
  > "Are you going to lock me up?" -> THALE: On what grounds? There is no one to charge.

THALE: Every morning, I am brought a sum: what was entered yesterday, less what is missing today. The sum has been wrong for twenty-two years. It gets more wrong.

THALE: You can remember what the rest of us cannot. Bring me what you remember, and I will treat you as a citizen, whatever the ink does.

  > "A village called Little Penance is gone." ->

[He writes this down. The words fade from the sheet as he finishes.]

THALE: I see. Say it again, please.

[He writes it again. It fades. He sets down the pen and squares it against the edge of the desk.]

THALE: Thank you. That is the eleventh this month.

### The Index Stacks

Odile takes the player to the College's lowest archive. Provost Sable objects at length. The puzzle is to find what is missing by looking for gaps: shelves with spaces no one can account for, and catalogue cards that refer to nothing. SEEN is not available yet, so players use the catalogue cross-references to work out each gap.

SABLE: These are the permanent collections. They are complete.

ODILE: Then why does it say "see also" on a card that isn't there, Emeric?

The Stacks are guarded by **the Indexer**, an automaton of card drawers. Its attacks fire cards, file the player into drawers, and alphabetize the room.

INDEXER: TITLE NOT FOUND. TITLE NOT FOUND. PLEASE RETURN ALL MATERIALS.

Behind it, the player finds a single card:

TEXT: UNENTERED, THE. See: VERSO. See also: [struck]. Two Standing Pages hold the reading of it. One keeps its roots. One keeps its salt.

ODILE: Well, one's in the Foxing Wood and one's under the sea. Nobody's ever accused the old scribes of considering the reader.

FLAG: faction_intros_available

## Q5: What the Wood Keeps

**Location:** The Foxing Wood. **Unlocks:** HIDDEN, and access to Heartwood Archive.

[Tannin. Madam Alder Rue receives the player at her long table. She is exquisitely polite.]

RUE: You are welcome here. Do sit. You will forgive my asking, but are you, by any chance, a relation of the Hollises?

  > "I don't know who I'm related to." -> RUE: How restful for you.
  > "Who are the Hollises?" -> RUE: A family who felled the Queen's Elm. Quite recently. In Line 3,212.

Sorrel Longmemory guides the player to the Heartwood Archive, a library grown as rings within an enormous living oak. Each ring holds a year.

SORREL: The Wood remembers what people forget. It can't be scraped, not all at once. There's too much of it, and it's too slow to notice.

Within the Archive, some rings are thin and white, recording the Erasure. Sorrel can recite the names of the villages lost.

SORREL: My people say them each morning. It takes eleven minutes. It took four, when I was young.

The player finds the Page of the Thicket (HIDDEN) and the first half of the reading.

TEXT: What is written has two faces. The ink goes through.

A Hollis descendant, Burdock Tew, can be exposed or protected. If the player protects him, Rue remembers, and Hollis Reprisal begins (side quest 14).

## Q6: The Drowned Column

**Location:** Saltmargin. **Unlocks:** WET or LIGHT, and the second half of the reading.

[Deckle Quay. Nan Twice demands the Wreck Tax in advance, on the grounds that you will probably sink.]

NAN: Could be worse, love. Could be you'd paid it already.

Old Dry Peter takes the player out in his ferry. He cannot swim, and has never considered this relevant. He looks at the player a little too long when they board.

PETER: Rowed someone south, a while back. Quiet sort. Didn't know where they were going. Same as you, mind.

  > "Was it me?" -> PETER: Couldn't say. Fare's the same.
  > "Who?" -> PETER: Told you. Quiet sort.

The Drowned Column is a tall sunken spire of the old capital. The player learns WET at the Tideline Page, which allows flooded locks to be drained or refilled. Drowned Parishioners keep attending a service that ended long ago.

DROWNED PARISHIONER: - and also with you - and also with you -

At the top of the sunken nave, the player finds the second half:

TEXT: Tear the page and you may walk the back of it. All that was scraped has left its mark there, in reverse.

## Q7: Over Scrivenhold

**Location:** Scrivenhold. **Boss:** Obelus. **Reward:** ABSENT. **Ends Act One. Triggers Erasure Stage 1.**

[On the player's return, the bells of Scrivenhold are ringing. Every Guard is on the walls. For the first time, Obelus has come for the capital.]

Battle across the rooftops and walls. Players must use Emendation to bring it down: drop bridges with LIGHT and HEAVY, light signal fires, and restore struck segments of the wall before they vanish. Obelus's touch rules a line across the arena that erases whatever crosses it. Marshal Hester Quillon holds the Upper Hand. Inter Pell's people carry children across the rooftops.

QUILLON: Hold the line! You, the clerk's mistake, get up that tower!

[Obelus falls and tears apart into long black strokes. One word remains, drifting.]

TEXT: ABSENT.

[Across the city, everything Obelus struck out writes itself back, line by line: wall, roof, bell tower. People stand in the street and watch their houses return.]

[Where it fell, the air has torn. A long vertical rip shows white letters on black paper, all backward.]

ODILE: Well. That's the door. Do you know, I've never been sure it was real.

THALE: You killed it.

  > "It can be done." -> THALE: It was one. There are others. Tomorrow's sum will still be wrong.

[A messenger arrives with a sheet. Thale reads it, and does not change expression.]

THALE: The northern line has moved. By eleven miles, overnight. I am told I had a cousin there. I am told this by my own handwriting.

FLAG: act_one_complete
FLAG: erasure_stage_1
