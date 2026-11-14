# Defending Earth — Status

Updated 2026-11-13. Where the work stands, so anyone (or any later
session) can pick it up. Plans are in `ROADMAP.md` (phase 1 has a numbered
work order), the code is explained in `CODE_MAP.md`, look in `ART_BIBLE.md`,
story in `LORE.md`, names in `NAMES.md`.

**Right now:** graphics phases 1 and 2 are finished. Phase 3 is under way: flashlights and flares are built; fire, glow and the skies are next (see `HANDOFF.md`). Lore: land roster settled and creature names
final; sea campaign deferred by the user until the land campaign is much
further along. A second and a third lore session were held on 2026-11-13
(see the two notes below). The third closed most of the open questions and
added the soldier pool. None of the third session is built.

**2026-11-13:** the test bench (a no-window way to run battles and print
reports) is now package 0, the first systems package, and the scale sheet
(one sheet of real-world sizes) opens graphics phase 4. Neither is built.
See `ROADMAP.md`.

**2026-11-13:** an architecture review ran and six rule fixes were decided (see `LORE.md`, "Fourth session"). No code has changed.

**2026-11-13:** the architecture review's designs for dice, shot odds, battle rules and saves are decided and recorded (`LORE.md`, "Fifth session"); an animation list is proposed in the roadmap; no code has changed.

**2026-11-13:** a sixth lore session decided the character designer for the soldier pool (hand-driven sliders first, optional photo later, no photo stored), ribbons with hidden citations, and footprints (`LORE.md`, "Sixth session"). This replaces the one-picture-per-soldier plan. No code has changed.

**2026-11-13:** a seventh lore session decided six ways for players to find their own stories: ribbon history, bonds, losing a bonded partner, a moment of the battle in each debrief, a memorial wall, and deployments for pool soldiers (`LORE.md`, "Seventh session"). Nothing is built; no code has changed.

**2026-11-13:** package 0 step 1 was built: three kinds of dice with a roll log (`src/dice.rs`), and one shot-odds rule for the shown and rolled number with height, range and cover. Builds; 22 unit tests pass; the self-test passes (re-run after range and cover were added). Not committed yet. The ROLL LOG screen, the seed on the new-campaign page, saves, IRON MAN MODE and FIXED DICE are not built.

**2026-11-13 (after the eighth session):** friendly fire and shielding were decided and are not built. A miss can now hit anyone or anything near its path, with no warning on screen; the shooter may gain a bad trait, the victim may lose trust, and the squad takes a morale hit. Shielding counts toward GUARDIAN and bonds. This changes two earlier decisions (`LORE.md`, "Eighth session, later part"). No code has changed.

**2026-11-13 (ninth session):** slice 1 of the battle rules is built: the shot rules return events and need no window. It has stray shots, cover hits, scenery damage, friendly-fire morale and the 50 to 150 percent damage roll. 33 unit tests pass and the self-test passes all three steps. Not committed yet. See `LORE.md`, "Ninth session".

**2026-11-13 (tenth session, the design fan-out):** the first round of design jobs ran through the `claude` command line, judged by eye by the lead; the owner agreed with every verdict. Built: the scale sheet (`source_assets/scale/`), flashlights and flares in battles (graphics phase 3, first two of five parts), and first models of eight creatures (`source_assets/blender/creatures/`, verdicts in `VERDICTS.md` there). 33 unit tests and the self-test pass. Thirteen jobs were not started; their briefs are in `docs/briefs/`. `docs/HANDOFF.md` says what to do next.

## Third lore session (2026-11-13)

- **Held and written down.** The decisions are in `LORE.md` under "Third
  session (2026-11-13): the open questions".
- **Nothing from it is in the game.** No code, rule, number or picture
  changed. It is all design on paper.
- **Questions it closed:**
  - *Soldier traits.* All eight starter traits are strengths, each with a
    built-in catch. Flaws are fears. A recruit gets one strength and one
    fear.
  - *Which fears.* The dark, fire, tight spaces, the Glow, water and
    sharks. Water and sharks do nothing on land, so they sleep until the
    sea campaign exists.
  - *Portraits.* Six stages, from new recruit to the thousand-yard stare.
    On top come marks from events: scars from wounds, and changes that come
    with earned traits.
  - *The ending.* There is no pass mark. Every living freed Tender the
    player brings to the Moon weakens the Seed's defences. Very few makes
    the last mission nearly impossible; many makes it fair.
  - *How the Maul is controlled.* A control spike, a piece of Registrar
    hardware driven into its molten core. Destroy the spike and the Maul
    goes wild and attacks whoever is nearest, the enemy included.
  - *Maul and Cinder.* Cinders are wild: nothing controls them, and they
    eat the enemy's dead as readily as ours. A Cinder grows as it eats, but
    it only becomes a full Maul between battles, never during one.
  - *How the Cinder looks.* A glowing, cube-shaped rock that rolls around,
    almost cute. Its cube and the Registrar's cube sign must never be
    confused (the rule is in `ART_BIBLE.md`).
  - *Spotting a buried Gorger.* Shifting ground and dust that a careful
    soldier can see from a few tiles away. The handheld radar finds one
    reliably.
  - *The human worker.* It first appears after the Registrar has read one
    of the player's soldiers. Early batches carry no weapon: they run at
    the squad, cling, and call out in broken recorded lines. Late batches
    hide among civilians and use human weapons.
  - *Refugees as soldiers.* A Regent uses Reach and cannot use human
    weapons. A Tender is small and frail, a scout that senses Reach and
    enemy machines. A Lash is the best marksman the player can have, but
    its first shot at any target costs extra time (it asks pardon). None
    can wear human armour; the workshop makes each kind its own harness.
  - *Mess hall.* Dead Gorgers stock it for refugees. Unfed soldiers lose
    morale and recover more slowly. Unfed refugees weaken, then leave.
    Nobody starves to death on screen.
  - *Infection inside the base.* Low risk brings sabotage: a module goes
    dark, research is lost, a craft is grounded. High risk brings a battle
    that starts inside the base with no warning. The Scrub Room pushes the
    risk back down.
  - *Signpost missions.* Four more, so five with "The same face": FIRST
    WORDS (first live capture), THE SPLIT VOTE (first Regent studied),
    ROLL CALL (a late batch of human workers appears) and THE OPEN DOOR
    (infection risk first runs high).
- **New: the soldier pool** (the user's idea).
  - A page reached from the game's starting screen. The player creates
    soldiers there once, with pictures. (Sixth session: made with a
    character designer, photo optional and not stored.)
  - Pool soldiers are mixed at random into the recruits of every campaign
    started afterwards. A "start with me" tick guarantees a place in the
    starting squad.
  - The player makes the name, look and picture (sixth session: the name
    and the designer's sliders). The strength and the fear
    are dealt fresh in every campaign and hidden like anyone else's.
  - One picture per soldier. The game lays the six portrait stages over it.
    *Changed 2026-11-13 (sixth session): the portrait is now a picture of a 3D head built from slider values, and the stages apply to that head. A photo is optional and not stored.*
  - Pool names and slider values stay on the player's machine. They are
    never put in the public repository. No picture is stored.
  - It is not covered by packages 1 to 9 of the game systems track, so it
    needs a package of its own in `ROADMAP.md`.
  - Not built. Today the game has no save file (it only writes screenshots
    and test results), has no box to type a name into, and does not read
    picture files from the player's disk (its own pictures are packed
    inside the program).
  - A risk to handle when it is built: screenshots go to `shots/`, which
    is in the public repository, so a private face could end up there.
    Suggested, not yet agreed with the user: keep the pool switched off
    whenever the game is started with `--demo`, `--screenshot`,
    `--self-test` or `--ai-test`.
- **Still open after the third session** (each is the user's call):
  - *Numbers, left for play.* `LORE.md` says these are set with the test
    bench, then checked by playing, once the systems exist: Slag from a full
    or an empty Maul, how fast it eats, and the hunger and infection figures.
  - *Not yet given a number or a list:* what each strength is worth; how
    many missions each portrait stage takes; how many batches of human
    worker there are; how many Gorgers a refugee needs; what scrubbing
    costs; from how many tiles a soldier spots a buried Gorger.
  - Which earned traits exist beyond shell shock and competence.
  - Multiplayer: what a joined player controls and gains; how players tell
    real chat from the Registrar's; soldiers crossing between games.
  - The order of the reveal that nobody is home, and of the name.
  - Whether the many-legged marksman picture was meant for the Lash.
  - How the Bloat looks (no picture yet). The Cinder now has a description
    but no picture.
  - Enemy craft, and the look of the Seed and the Root.
  - Waiting with the sea campaign: the sea roster and the sea Regent
    details.
- **New questions the third session raises.** These came out of checking
  the answers against the code. They are not in `LORE.md` and the user has
  not been asked yet. None is decided:
  1. How does the player aim at the Maul's spike: by shooting it from
     behind, or with a special aimed shot?
  2. Do living wild units (a wild Maul, Cinders) stop a battle from ending?
  3. What does a Cinder that survives a battle do between battles?
  4. A copy wears the face of the soldier it was read from. May a copy wear
     a pool soldier's real photo? (Sixth session: no picture is stored,
     only slider values, so this mostly falls away.) And may a pool
     soldier's head (built from slider values) ever be shown in another
     player's game? (`ART_BIBLE.md` says a soldier's record can later be
     shown in another player's game. `LORE.md` says pool data stays on the
     player's machine.)
  5. Does the fear of the Glow react to a soldier's own cored weapon (its
     core is filled with Glow)?
  6. The photo step (10c): the game can read PNG files today but not JPG,
     which is what phone photos are. Add JPG, or accept PNG only? (Sixth
     session: a photo is optional, read once and thrown away; still to
     settle.)
  7. Does a freed Tender count for the ending if it is alive at base, or
     only if it is aboard the transport to the Moon?
- **Documents not yet in step.** `LORE.md` keeps its older "Still to
  decide" list, which still shows questions this session closed. Where the
  two disagree, the "Third session" section wins.

## Second lore session (2026-11-13)

- **Held and written down.** The decisions are in `LORE.md` under "Decided
  in the second session".
- **On-screen words renamed in the game.** Only the words changed. No rule
  or number changed.
  - Recovered Lattice is now SPINE. Electrolyte is now GLOW. The Lattice
    Carrier armour is now the SPINE CARRIER.
  - Psi is now REACH everywhere on screen (REACH DAMPER, REACH WING, REACH
    STUDIES, the battle button). The key is still P.
  - Derived Battery is now BREACHER BATTERY.
  - The final land site moved from Mars to the far side of the Moon. The
    map label, the research topic (THE FARSIDE SIGNAL), the win screen and
    the Albatross text all say so.
  - The Heron is described as a twin-rotor transport helicopter.
  - Inside the code the old words remain as internal names (for example
    `lattice`, `electrolyte`, `psi_strength`, research id `SEED_LOCATION`).
    The player never sees them.
- **A game systems track now exists in `ROADMAP.md`.** It is nine packages
  of rules work, each matched to a graphics phase. Nothing in it is built
  yet.
- **Decided but not built yet:**
  - The SLAG material, and collecting Glow from pools.
  - MESS HALL, SCRUB ROOM and REFUGEE QUARTERS buildings.
  - SHOTGUN and KNIFE.
  - Removing the Trestle unit. It is still in the code.
  - New rules for the Maul, Brooder (the taker), Lathe (the builder), Kite,
    Bloat, Cinder and Gorger.
  - Soldier traits: one strength and one flaw, hidden until triggered,
    eight starters, earned traits, and fears. Portraits that change with
    service. (The third session settled the details: see above.)
  - Freed aliens ("refugees") as soldiers.
  - Infection tax and scrubbing.
  - The human worker and its improving voice lines ("duck calls").
  - Signpost missions.
  - The Moon finale: low gravity, real lunar surface maps, dark-side
    lighting, the freed Tenders' vote and merge, and the last word that
    there are many.
  - Multiplayer: join or be wiped out.
- **Still open after the second session.** This is the list as it stood
  then. The third session closed most of it; the current list is in the
  third session note above.
  - Which of the eight starter traits are strengths and which are flaws,
    and their numbers. Which fears exist beyond water and sharks. Which
    earned traits exist beyond shell shock and competence.
  - Maul: how the Registrar reads and controls it, SLAG yield for a full
    versus an empty corpse, how fast it eats, how it overlaps with the
    Cinder.
  - Mess hall: what happens when it runs out, how many Gorgers a refugee
    needs, and whether they must be dead or alive.
  - Infection tax: the numbers, what scrubbing costs, what an inside
    attack looks like.
  - What each freed species can and cannot do as a soldier.
  - The human worker: its look, when it first appears, its signpost
    mission.
  - More signpost missions and their triggers. The order of the reveal
    that nobody is home, and of the name.
  - Whether the many-legged marksman picture was meant for the Lash.
  - How the Bloat and the Cinder look (no picture for either yet).
  - Enemy craft, and the look of the Seed and the Root.
  - Multiplayer: what a joined player controls and gains; how players tell
    real chat from the Registrar's; soldiers crossing between games.
  - Waiting with the sea campaign: the sea roster and the sea Regent
    details.

## Done

- **0.9 identity swap.** All names, creature roster, item tables and docs
  are ours. Release build clean; `--self-test <file>` passes; leftover-name
  sweeps clean. The two old data files and the backup from before the
  swap were deleted on 2026-11-13.
- **Globe** is the right way up (`src/globe.rs`, the screen y flip in the
  fragment shader).
- **Lighting lab** (`examples/lightlab.rs`, using `src/gfx.rs` and
  `src/sky.rs`, which the game now uses too):
  - Shadow pass (4096, soft edges) → HDR scene pass → overlay pass (smoke,
    fire, tracers, markers) → auto-exposure, bloom, lens.
  - Sun and sky colours for dawn, morning, noon, afternoon, golden hour and
    dusk are measured from Blender atmosphere renders
    (`source_assets/blender/sky/`, table in `assets/sky/looks.txt`). Night,
    overcast and storm are still set by hand.
  - Ground: colours and field types from a per-site map; photographed grain
    from five CC0 Poly Haven surfaces (`source_assets/polyhaven/`, packed by
    `make_ground.py` into `assets/ground/`), laid so it never visibly
    repeats.
  - 5.5 ms per frame at 1920×1080 (the screen's 180 Hz cap).
  - Run: `cargo build --release --example lightlab` then
    `DISPLAY=:0 ./target/release/examples/lightlab --view tele --clock "2026-06-21 8am" --frames 40 --screenshot out.png`
    (views: wide, default, tele, site, low; also `--cover`, `--storm`,
    `--wet`, `--seed`).
- **Graphics phase 1 (2026-11-13).** Battles draw through the lab's
  lighting: real sun and shadows from the mission's date, time and place,
  long-lens camera, photographed ground grain, apron of country around the
  map, glow parts in their own pass. 5.6 ms per frame at 1920×1080 (the
  180 Hz cap) in every view tried. Self-test passes
  (`checks/selftest_phase1.json`). The globe's sun follows the campaign
  clock and real city lights come on as night passes, big cities first.
  Debug flags are listed in `CODE_MAP.md`. Best view:
  `--clock "2027-06-21 7am" --weather clear --view tele --demo tactical`.
  Screenshots: `shots/phase1_*.png`.
- **Graphics phase 2 (2026-11-13).** Weather is drawn in 3D: rain streaks
  with splashes and puddles, snowfall with blowing drift, dust; heavy
  versions `storm`, `blizzard`, `duststorm`. Snow lies on ground, roofs,
  trees and crates; sand has wind ripples. All seven land biomes in the
  world map draw (temperate, farmland, mountain, rain forest, desert, ice,
  northern forest) and follow the season. `--site lat,lon` picks a real
  place. The battle header names the time of day (MIDDAY, EVENING and so
  on) and, in the tropics, WET SEASON or DRY SEASON. 5.56 ms per frame in
  every shot. Self-test passes. Best view:
  `--clock "2027-01-15 3pm" --weather clear --site 30,82 --demo tactical`
  (mountain in winter). Screenshots: `shots/phase2_r3*.png`.
  - **Game rule change, inherited bug fixed:** the world-map lookup was
    180° of longitude off, so every site got the land/water and biome of
    the far side of the planet. Fixed in `data::world_index`. Missions now
    get the right terrain for where they are.
  - Rain is drawn as 1/30 s streaks, not the art bible's 1/100 s shutter,
    because shorter streaks read as white specks from this height.
- **City lights map** for the globe: `assets/earth_lights.png`, built from
  NASA's Earth at Night (`source_assets/nasa_black_marble/`).

## Next, in order

1. **Read `docs/HANDOFF.md`.** It lists the thirteen design jobs not yet
   run, in order, and what is waiting on the owner.
2. **Graphics phase 3, the rest:** fire (G3), glow (G4), then night,
   overcast and storm skies (G5). Flashlights and flares are built.
3. Phases 4–9 as listed in `ROADMAP.md`, which now holds the full list of
   things to design and animate and which AI model does which part.
4. The game systems track in `ROADMAP.md`, alongside the graphics phases.
   Package 1 (groundwork) needs no art and can go with phase 3.
5. Lore: the open and new questions listed under "Third lore session"
   above.

## Known flaws

- Weather: puddles read as pale stickers, they do not mirror the sky;
  storm and dust scenes are flat and low in contrast (they need the real
  overcast and storm skies); falling snow is sparse; crop rows are too
  regular and striped; the grass apron outside the map looks rippled like
  water; hours from 6pm to 9pm are labelled EVENING whatever the sun does.
- Lighting: night is lit only by flashlights and flares; trees, buildings,
  crates and soldiers are still boxes; tree tops glint white at low sun;
  cliff edges show as dark zigzag walls; the mouse was not tested by hand
  against the new camera (name tags line up, and they share its maths).
- Lights (phase 3 so far): a burning hand flare takes all four lamp
  shadows, so flashlights cast none while it burns; the debug sky flare
  never burns out; the cost per frame of flashlights and flares has not
  been measured; pictures of flashlights in a dusk dust storm and at noon
  were not taken; the old glowing ground square still draws under
  `--lights off` (keep or delete: the owner's call).
- Creature models are not in the game. The game cannot load a Blender
  model yet (see "The model loader" in `ROADMAP.md`).
- `--clock` on the globe moves the sun but not the date shown in the header.
- Game (from the 0.9 check): punctuation after Y, P, w or y disappears in
  small text; the target panel covers the controls hint in the field view;
  a few unsold items sell at a flat $700; sea soldiers show COMBAT UNIFORM
  at base.
- Game (records): a soldier's mission count never goes up, wounded soldiers
  never actually rest, and every won mission gives the same flat loot (25
  Spine and 10 Glow). The game systems track fixes these: the records in
  package 1, the loot in package 2.
- The Moon is not drawn. The final assault's marker sits on Earth and the
  battle is an ordinary one. The win screen says the Seed is dead, but the
  decided ending is a vote (the Registrar cannot be killed).
- `mechanics.json`, `NAMES.md` and `CODE_MAP.md` may still use the old
  words until they are brought in step.
- Battles still get the old stepped screen-edge darkening and scan lines
  drawn over them (`ui::vignette`, `scanlines` in `src/main.rs`), on top of
  the new lens.
- The in-game picture of the rules has no weather or night effect: sight
  stays the same in a blizzard or in the dark.

## Working notes

- The repository is https://github.com/iannitram/defending-earth (public, created
  2026-11-13). The project folder `dev/Defending Earth` is the repository; `target/` is
  left out. Older screenshots
  and the first two rounds of phase 2 were deleted. The git history is now
  the only backup; the first commit is the state after phase 2.
- The game needs `DISPLAY=:0` to open a window from a script. Every launch
  shows on the user's screen for a moment.
- The old investigation folders and backups were deleted on 2026-11-13 at
  the user's request, and the game was moved up out of its `app/` folder,
  so the project folder holds only the game.
- `src/main.rs` sets the draw buffer with
  `gl_set_drawcall_buffer_capacity(65_535, 400_000)`. The first number must
  stay at or below 65,535 (the most points one draw batch can address), or
  shapes come out scrambled.
- Blender 5.3 (a Flatpak) is driven by script; scenes `DE_bake_test` and `DE_sky_bake`
  are ours, `Scene` is the user's.
