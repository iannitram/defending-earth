# Defending Earth — Status

Updated 2026-11-13. Where the work stands, so anyone (or any later
session) can pick it up. Plans are in `ROADMAP.md` (phase 1 has a numbered
work order), the code is explained in `CODE_MAP.md`, look in `ART_BIBLE.md`,
story in `LORE.md`, names in `NAMES.md`.

**Right now:** graphics phases 1 and 2 are finished. Next is phase 3
(flashlights, flares, fire). Lore: land roster settled and creature names
final; sea campaign deferred by the user until the land campaign is much
further along. A second lore session was held on 2026-11-13 (see the note
below).

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
    eight starters, earned traits, and fears (afraid of water, afraid of
    sharks and the like). Portraits that change with service.
  - Freed aliens ("refugees") as soldiers.
  - Infection tax and scrubbing.
  - The human worker and its improving voice lines ("duck calls").
  - Signpost missions.
  - The Moon finale: low gravity, real lunar surface maps, dark-side
    lighting, the freed Tenders' vote and merge, and the last word that
    there are many.
  - Multiplayer: join or be wiped out.
- **Still open** (from `LORE.md`; each is the user's call):
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

1. **Graphics phase 3:** flashlights, flares, fire.
2. Overcast, storm and night skies from Poly Haven HDRIs (downloads need the
   Tailscale exit node on).
3. Phases 4–9 as listed in `ROADMAP.md`, which now holds the full list of
   things to design and animate and which AI model does which part.
4. The game systems track in `ROADMAP.md`, alongside the graphics phases.
   Package 1 (groundwork) needs no art and can go with phase 3.
5. Lore: the open questions listed under "Second lore session" above.

## Known flaws

- Weather: puddles read as pale stickers, they do not mirror the sky;
  storm and dust scenes are flat and low in contrast (they need the real
  overcast and storm skies); falling snow is sparse; crop rows are too
  regular and striped; the grass apron outside the map looks rippled like
  water; hours from 6pm to 9pm are labelled EVENING whatever the sun does.
- Lighting: night
  is close to black until flashlights exist (phase 3); trees, buildings,
  crates and soldiers are still boxes; tree tops glint white at low sun;
  cliff edges show as dark zigzag walls; the mouse was not tested by hand
  against the new camera (name tags line up, and they share its maths).
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
- Blender 5.2 is driven by script; scenes `DE_bake_test` and `DE_sky_bake`
  are ours, `Scene` is the user's.
