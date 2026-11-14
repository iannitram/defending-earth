# Rules for every job (read all of this first)

You are one run in a larger piece of work on the game Defending Earth
(Rust, macroquad). The project folder is `/home/deck/dev/Defending Earth`.
A lead has already made every design decision. You carry the brief out.

- **Do not ask questions.** Nobody is there to answer. If the brief does not
  cover a choice that changes what the result looks like, what it is called,
  or how a rule works: **stop**, change nothing further, and end with a
  report that starts with the line `UNDECIDED:` and states the question.
  Do not guess. Small technical choices (variable names, mesh density, how
  to call a Blender function) are yours.
- **Write only the files listed under "Files you may write".** Read anything
  in the project. Never write anywhere else in the project.
- **Never commit, push, stash, reset or otherwise touch git.**
- **Never launch the game** (`defending_earth`, `lightlab`). The game opens
  a window on the owner's screen. Only the lead launches it.
- **Never use a sub-agent, the Agent tool or a workflow.** Do the work
  yourself in this run.
- **Never use the Blender MCP tools or connect to an open Blender.** Blender
  runs only in the background from the shell (see the Blender section if
  your job has one).
- **Original work only.** Never name, copy or allude to an existing
  commercial game, film or real military unit. No real flags, badges,
  camouflage patterns, maker's names or real people's names or faces.
- **The repository is public.** Never write personal details, keys or
  passwords into any file.
- Plain words in every document you write: short sentences, no jargon
  without a one-line explanation. Times are written like 7am and 4:45pm.
- Pictures in `reference/` and `docs/lore_refs/` are idea references only.
  You may look at them. Build from the description in this brief, never
  copy a picture.

## How to finish

End with a short plain report, in this order:
1. `DONE`, `PARTLY DONE` or `UNDECIDED:` on the first line.
2. Every file you wrote, one per line.
3. What you checked and the result (commands run, numbers measured).
4. The three weakest things in the result, honestly.
5. Anything the lead must decide.
# Code rules for this job

- Read `docs/CODE_MAP.md` (the parts on `gfx.rs`, `tactical.rs` and the
  debug flags) and `docs/ART_BIBLE.md` sections 1 to 4 before changing
  anything.
- Build with `cargo build --release` from the project folder. It must build
  with no new warnings. You may not run the built game: it opens a window
  on the owner's screen. The lead takes the screenshots and judges them,
  and may send the job back to you with notes.
- **Rules must not change.** Nothing you add may change a dice roll, a
  shot, movement, sight or the outcome of a battle. Lights and effects are
  drawing only in this job. Do not touch `src/dice.rs` or `src/data.rs`.
- Keep the engine (macroquad). If something cannot be done in macroquad,
  stop with `UNDECIDED:` and say what.
- The draw-call buffer size in `src/gfx.rs` must stay at or below 65,535.
- Light limits (art bible): at most 4 lights that cast shadows and 24
  other moving lights on screen at once. Frame budget 16.6 ms at
  1920×1080.
- Brightness scale already in the code: noon sunlit white is about 11,
  fire about 6, a tracer about 20. Exposure adapts, so at night a small
  light looks bright. Light sources blow out to white at their core;
  bloom only on the few brightest things, never over the whole picture.
- Not wanted: cartoon fire, glowing outlines, rim light, flat ambient
  fill, even light with no falloff.
- Every effect gets a debug switch so the lead can photograph it. Add the
  switch named in the brief, read with the other debug flags, and write
  one line for it in the debug flag list in `docs/CODE_MAP.md`.
- In your report give the exact command lines the lead should run to see
  each thing (for example `DISPLAY=:0 ./target/release/defending_earth
  --demo tactical --clock "2027-06-21 11pm" --fx flare --screenshot
  /tmp/x.png`). Do not run them.

## Added after the first lights job (overrides anything below that differs)

- **Brightness at night.** The first lights job showed that the night
  exposure is far more sensitive than the brief assumed: a flashlight
  set to 2.5 on the brightness scale burnt the whole picture white, and
  0.07 looks right (`FLASH_POOL` in `src/tactical.rs`). So every
  strength in your job's text that is given "on the code's brightness
  scale" for LIGHT FALLING ON THE GROUND must be divided by 35 before
  use. Values for a glowing surface or core that is meant to blow out
  to white stay as written.
- Put every strength, reach, flicker and size number of your job as a
  named constant in one block with a one-line comment each, so the lead
  can tune them after looking at pictures. List them in your report.
- `Lamp` in `src/gfx.rs` (`Lamp::cone`, `lamp.start`, `lamp.falloff`,
  the 4 lamp shadows, `set_air` for beams in weather) is finished and
  accepted. Use it as it is; do not change how it works or the
  flashlight numbers.
# Job G5: overcast, storm and night skies measured from sky photographs

## Goal

The battle sky has nine states. Six (dawn, morning, noon, afternoon,
golden hour, dusk) are measured from Blender renders. Three (overcast,
storm, night) are still numbers set by hand. Replace those three with
numbers measured from real sky photographs.

## The settled design

- **The photographs** are three CC0 sky pictures from Poly Haven (free
  for any use). Use exactly these, at 2k size in EXR form:
  - overcast: `overcast_soil_puresky`
  - storm: `mud_road_puresky`
  - night: `kloppenheim_02_puresky`
  Find each file's address through `https://api.polyhaven.com/files/
  <name>` (use `curl` with the user agent `asset-fetch`; send nothing
  about the user or this project). Save them in
  `source_assets/polyhaven/skies/` with a `SOURCE.md` that gives the
  name, address, date (2026-11-13) and licence of each.
- **Measure them the way the clear skies were measured.** Read
  `source_assets/blender/sky/make_looks.py`,
  `source_assets/blender/sky/clear_states.json` and the functions
  `baked_look`, `state_look` and `state_look_hand` in `src/gfx.rs` to see
  which numbers a sky state needs and how they were taken. Write
  `source_assets/blender/sky/make_photo_looks.py` that takes the same
  numbers from the three photographs (run it with Blender in the
  background: `flatpak run org.blender.Blender -b --python <script>`,
  or with Blender's bundled Python) and writes
  `source_assets/blender/sky/photo_states.json`.
- **Adjustments, exactly these:**
  - Overcast: as measured, scaled so that its overall brightness equals
    the hand-set overcast state's brightness today. Only the colours and
    the balance between top of sky and horizon change.
  - Storm: the measured numbers at 30 percent of the overcast
    brightness, with red multiplied by 0.90 and blue by 1.08, so it is
    dark and cold.
  - Night: the measured colours, scaled so that overall brightness
    equals the hand-set night state's today at full moon. The existing
    rule that the moon's phase dims the night stays exactly as it is.
- **In the game:** make the three states read the measured numbers, in
  the same way the six clear states read theirs. Keep the hand-set
  function as the fallback it is today.
- Add one line to `CREDITS.md` under the Poly Haven entry naming the
  three skies.

## Files you may write

`source_assets/polyhaven/skies/*`, `source_assets/blender/sky/
make_photo_looks.py`, `source_assets/blender/sky/photo_states.json`,
`src/gfx.rs` (only the sky state numbers and the code that reads them),
the Poly Haven entry in `CREDITS.md`, and the sky notes in
`docs/CODE_MAP.md` and section 4 of `docs/ART_BIBLE.md` (only the
sentence that says night, overcast and storm are set by hand).

## Done looks like

- `cargo build --release` succeeds with no new warnings.
- The report shows a small table: for each of the three states, the old
  hand-set numbers beside the new measured ones.
- The report gives command lines for the lead to photograph each state
  (overcast noon, storm afternoon, clear night at full moon and at new
  moon).

## Must not touch

The six clear states, any rule, weather choice, anything else.


## Added after the flares job

- Coloured lights wash out to orange-white on this ground. A red light needs a strongly red colour (the hand flare uses 1.0, 0.13, 0.09). Do the same for any coloured light: push the colour further than looks right on paper.
- A light only 0.25 m above the ground skims it and dies within 3 m. Lights that must reach across ground sit at least 0.8 m up.
- The camera brightens any evenly lit night until it looks like a dull day. While a sky flare lights the night the camera is held back 1.5 stops (`SKY_NIGHT_STOPS` in `src/tactical.rs`). Do not change that; do not add another whole-map light without a matching hold-back, and report it if you do.
- The flare constants in `src/tactical.rs` were tuned by the lead by eye. Do not change them.
