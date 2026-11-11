# Section 2: art direction

## 2. Art direction: a real world, written small

Vellmark looks like a real place. Stone, timber, snow, skin, and water are lit and shaded naturally, in the manner of Skyrim: a northern realm of weather, long views, firelight, and dusk. It is grounded and atmospheric, and it does not chase the detail of current big-budget games.

Then the player walks up to a wall. Within a few steps its surface turns out to be made of thousands of tiny characters, like the code under the world. Closer still, they can be read.

**The core idea: the world is written in fine print.** Every surface is composed of letters far too small and too many to see from any distance. From afar they average into ordinary colour and light, and nothing looks unusual. The letters appear only near the camera. A player should spend minutes in the game before noticing, and never see the world the same way afterwards.

**Three distances, each a designed experience:**

1. **Far and mid (beyond about 6 m):** a naturally lit, realistic world. No individual symbol can be made out, and there is no visible grid, grain, or shimmer. A screenshot at this distance must pass as a conventionally rendered game.
2. **Near (about 6 m down to 2 m):** a fine weave appears in surfaces, first as texture and then as recognisable characters. Larger characters turn out to be built from smaller ones.
3. **Arm's length (under about 2 m):** the finest layer is readable. Walls, clothing, gravestones, and bark contain real text.

The distance at which letters begin to show is the **reveal distance**. Expose it and the letter contrast for tuning at the Art Gate.

**Close reading shows the code.** Holding Read (Section 5h) drops the realistic image and shows the whole view as text at every distance. This is the only time the world's writing is visible from afar, apart from the places below.

**Where the text breaks through.** A few things show their letters openly, and they matter more because nothing else does:

- **Emendation.** An Inscription is fine gold lettering on its target that brightens when aimed at. A written word flies as gold characters, and the changed surface flashes its letters before settling.
- **The Erasure.** As the world is scraped, its letters grow coarse and sparse, so the Erasure is the one region whose writing can be seen from a distance (Section 2a).
- **The Verso.** White letters on black, plainly visible, and backward.
- **Damage and death.** Letters are knocked from a body by hits, and a slain creature collapses into a pool of characters.
- **Gold leaf.** Sealed words and sacred things carry gold lettering with real specular glint, as in an illuminated manuscript.

**Beauty rules:**

- **Light first.** Sun, sky, shadow, fog, and fire carry the image. Compose value and silhouette as a landscape painter would.
- **Natural colour, graded by place.** Each Quire has its own colour grade for each time of day. Where the canon gives a Quire a palette of paper, ink, and accent, read it as that Quire's grade and as the tints its letters take up close.
- **Crisp type, no smear.** Up close, letters are sharp. No temporal blur, film grain, or chromatic aberration.
- **No noise from afar.** Letters too small to resolve must vanish completely into the surface. Moiré or sparkle at any distance is a defect.
- **Letters as relief.** Up close, characters are cut slightly into stone and wood and raised slightly on cloth and skin, so they catch the light like carving and embroidery.
- **Dusk and firelight are the money shots.** Sunset, starry nights, firelit streets, glowing windows, and aurora must be breathtaking. Spend time here.
- **Silence and scale.** Compose vistas deliberately. Place landmarks that draw the eye over long distances.

**Glyph families (what each material is written in, tune by eye):**

| Material | Characters |
|---|---|
| Rock | `# % & @ 8` |
| Snow | `. : ' * ,` |
| Grass, moss | `/ \ \| , " '` |
| Timber | `\| ! l I 1` (following grain) |
| Masonry | `H E B [ ]` plus authored text |
| Water | `~ - = _` |
| Metal | `= + T Y` |
| Cloth | `x v w z` |
| Skin | `o e c a s` |
| Leaves | `^ A v` |
| Stars, seen in close reading | `. * +` |

**Material lexicons (what each material's letters may spell up close):**

Within the lattice and the fine print, a material often spells its own name and, more often still, what it is made of, scattered evenly and written exactly as its grain is. While a thing is hot or cold, its letters say so, glitching in and out: wood smokes, smoulders, then burns; iron glows, then melts; water steams, boils, and goes to gas, or freezes to ice. The words are data, in `assets/materials/lexicon.ron`, which is the truth for this table; a test fails if the two disagree. `H2O` is written only where water is what the thing is. Every word is printable ASCII. The Erasure has no words: it is being erased.

<!-- lexicon:start -->
| Material | Name | Made of | Hot, least to most | Cold, least to most |
|---|---|---|---|---|
| masonry | `stone` | `SiO2`, `CaCO3`, `Al2O3` | `HOT`, `CRACKING`, `GLOWING` | `COLD`, `FROSTED` |
| rock | `rock` | `SiO2`, `KAlSi3O8`, `Fe2O3` | `HOT`, `CRACKING`, `MOLTEN` | `COLD`, `FROSTED` |
| basalt | `basalt` | `SiO2`, `MgO`, `FeO` | `HOT`, `GLOWING`, `MOLTEN` | `COLD`, `FROSTED` |
| obsidian | `obsidian` | `SiO2`, `Al2O3`, `Fe3O4` | `HOT`, `GLOWING`, `MOLTEN` | `COLD`, `FROSTED` |
| lava | `lava` | `SiO2`, `FeO`, `MgO` | `FLOWING`, `SPITTING`, `BURNING` | `COOLING`, `SETTING` |
| sulfur | `sulfur` | `S8`, `SO2`, `H2S` | `HOT`, `MELTING`, `BURNING` | `COLD` |
| coal | `coal` | `C`, `S`, `CH4` | `GLOWING`, `SMOULDERING`, `BURNING` | `COLD` |
| ash | `ash` | `C`, `K2CO3`, `CaO` | `GLOWING`, `SMOULDERING`, `BURNING` | `COLD` |
| salt | `salt` | `NaCl`, `KCl`, `MgSO4` | `HOT`, `CRACKLING`, `MELTING` | `COLD` |
| brick | `brick` | `Al2Si2O5(OH)4`, `SiO2`, `Fe2O3` | `HOT`, `BAKING`, `GLOWING` | `COLD`, `FROSTED` |
| earth | `earth` | `SiO2`, `Al2Si2O5(OH)4`, `C` | `HOT`, `BAKING` | `COLD`, `FROZEN` |
| mud | `mud` | `H2O`, `SiO2`, `Al2Si2O5(OH)4` | `DRYING`, `CRACKING`, `BAKING` | `COLD`, `FROZEN` |
| gravel | `gravel` | `SiO2`, `CaCO3`, `Fe2O3` | `HOT`, `CRACKING` | `COLD`, `FROSTED` |
| sand | `sand` | `SiO2`, `CaCO3`, `Fe2O3` | `HOT`, `FUSING`, `MELTING` | `COLD` |
| peat | `peat` | `C`, `C6H10O5`, `C10H12O3` | `SMOKING`, `SMOULDERING`, `BURNING` | `COLD`, `FROZEN` |
| snow | `snow` | `H2O` | `THAWING`, `MELTING` | `FREEZING`, `FROZEN` |
| ice | `ice` | `H2O` | `THAWING`, `MELTING` | `HARDENING`, `FROZEN` |
| water | `water` | `H2O`, `O2`, `CaCO3` | `STEAMING`, `BOILING`, `GAS` | `FREEZING`, `ICE` |
| brine | `brine` | `H2O`, `NaCl`, `MgCl2` | `WARM`, `STEAMING`, `BOILING` | `COLD`, `FREEZING` |
| grass | `grass` | `C55H72MgN4O5`, `C6H10O5`, `C6H12O6` | `WILTING`, `SMOULDERING`, `BURNING` | `FROSTED`, `FROZEN` |
| moss | `moss` | `C55H72MgN4O5`, `C6H10O5`, `C6H12O6` | `DRYING`, `SMOULDERING`, `BURNING` | `FROSTED`, `FROZEN` |
| reed | `reed` | `C6H10O5`, `SiO2`, `C55H72MgN4O5` | `DRYING`, `SMOULDERING`, `BURNING` | `FROSTED`, `FROZEN` |
| straw | `straw` | `C6H10O5`, `C5H10O5`, `SiO2` | `SMOKING`, `CATCHING`, `BURNING` | `COLD`, `FROSTED` |
| leaves | `leaf` | `C55H72MgN4O5`, `C6H10O5`, `C6H12O6` | `WILTING`, `CURLING`, `BURNING` | `FROSTED`, `FROZEN` |
| fungus | `fungus` | `C8H13NO5`, `C6H12O6` | `WILTING`, `SHRIVELLING`, `BURNING` | `COLD`, `FROZEN` |
| bark | `bark` | `C6H10O5`, `C10H12O3`, `CaC2O4` | `SMOKING`, `SMOULDERING`, `BURNING` | `COLD`, `FROZEN` |
| timber | `wood` | `C6H10O5`, `C10H12O3`, `C5H8O4` | `SMOKING`, `SMOULDERING`, `BURNING` | `COLD`, `FROZEN` |
| iron | `iron` | `Fe`, `Fe2O3`, `C` | `HOT`, `GLOWING`, `MELTING` | `COLD`, `BRITTLE` |
| steel | `steel` | `Fe`, `C`, `Mn` | `HOT`, `GLOWING`, `MELTING` | `COLD`, `BRITTLE` |
| bronze | `bronze` | `Cu`, `Sn`, `Cu2CO3(OH)2` | `HOT`, `GLOWING`, `MELTING` | `COLD` |
| copper | `copper` | `Cu`, `Cu2O`, `CuCO3` | `HOT`, `GLOWING`, `MELTING` | `COLD` |
| silver | `silver` | `Ag`, `Ag2S` | `HOT`, `GLOWING`, `MELTING` | `COLD` |
| gold | `gold` | `Au` | `HOT`, `SOFTENING`, `MELTING` | `COLD` |
| lead | `lead` | `Pb`, `PbO`, `PbCO3` | `HOT`, `SOFTENING`, `MELTING` | `COLD` |
| glass | `glass` | `SiO2`, `Na2O`, `CaO` | `HOT`, `SOFTENING`, `MELTING` | `COLD` |
| cloth | `linen` | `C6H10O5` | `SCORCHING`, `BURNING` | `COLD`, `STIFF` |
| wool | `wool` | `C`, `N`, `S` | `SCORCHING`, `SINGEING`, `BURNING` | `COLD`, `STIFF` |
| rope | `rope` | `C6H10O5`, `C10H12O3` | `SCORCHING`, `FRAYING`, `BURNING` | `COLD`, `STIFF` |
| leather | `leather` | `C`, `N`, `C76H52O46` | `SCORCHING`, `CRACKING`, `BURNING` | `COLD`, `STIFF` |
| skin | `skin` | `C`, `N`, `O` | `WARM`, `SWEATING`, `BURNING` | `SHIVERING`, `FROSTBITTEN` |
| bone | `bone` | `Ca10(PO4)6(OH)2`, `CaCO3`, `C` | `SCORCHING`, `CHARRING`, `CALCINING` | `COLD`, `BRITTLE` |
| horn | `horn` | `C`, `N`, `S` | `SCORCHING`, `CURLING`, `BURNING` | `COLD`, `BRITTLE` |
| shell | `shell` | `CaCO3` | `HOT`, `CRACKING` | `COLD`, `BRITTLE` |
| paper | `paper` | `C6H10O5`, `CaCO3`, `C10H12O3` | `BROWNING`, `CURLING`, `BURNING` | `COLD`, `BRITTLE` |
| ink | `ink` | `C7H6O5`, `FeSO4`, `C` | `DRYING`, `BLISTERING`, `BURNING` | `THICKENING`, `FROZEN` |
| wax | `wax` | `C46H92O2`, `C25H52` | `SOFTENING`, `MELTING`, `BURNING` | `COLD`, `BRITTLE` |
| tar | `tar` | `C`, `C10H8`, `S` | `SOFTENING`, `BUBBLING`, `BURNING` | `COLD`, `SET` |
| stars | `star` | `H`, `He`, `C` | `BURNING` | `COLD` |
| erasure | none | none | none | none |
<!-- lexicon:end -->
