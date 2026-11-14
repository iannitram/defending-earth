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
# Job G3: fire (the look only)

## Goal

Draw fire in the battle view: flames, smoke, embers and the light it
throws, using the moving-light system that already exists in
`src/gfx.rs` (read it first). The look only. How fire starts, spreads
and hurts is a rule and is not part of this job.

## The settled design

Fire must pass as a still from aerial news footage. The failure to
avoid is cartoon fire: flame-shaped orange blobs with outlines.

- *Flames:* built from several thin sheets that always face the camera,
  each cut by a scrolling noise so it is ragged, rising and thinning
  toward the top, never the same twice. Colour by heat: white-hot at the
  base and centre (about 30 on the code's brightness scale, so it blows
  out to white), yellow above that, orange (about 6) at the edges, dark
  red only at the last wisps. A ground fire's flames reach 1.5 m. A
  burning tree or bush burns to one and a half times its own height.
- *Smoke:* dark grey-brown, drawn so it darkens what is behind it (not
  added as light). Lit orange from below near the fire and by the sky
  higher up. It rises at least 15 m, widens, and leans and drifts with
  the wind. Soft and layered, with no round puffs and no hard edges.
- *Embers:* a few small orange points that rise in the smoke, drift and
  go out.
- *Light:* one point light for each burning group, at most 6 on screen
  (merge neighbouring burning squares into one group). The light sits
  1.2 m above the ground, colour red 1.0, green 0.55, blue 0.22, reach
  16 m, strength chosen so that ground 3 m away is about 6 on the scale.
  Flicker 20 percent, changing 3 to 7 times a second, and the light's
  position wanders up to 0.3 m, so shadows and highlights move.
- *Ground:* burning and burnt squares are scorched; the code already has
  a scorch colour, use it.
- *By day* the flames look pale and thin and the smoke is the main
  thing. *By night* the fire is the picture: the automatic exposure
  should let its surroundings fall dark.
- Limits: keep the count of flame and smoke sheets low enough that a
  3 by 3 patch and a tree burning together cost under about 2 ms.

**Debug switch.** Add `fire` to the `--fx` list (create `--fx`, a list
separated by commas, if it does not exist yet): a 3 by 3 patch of
burning ground five squares from the middle of the map, and the nearest
tree or bush to it burning.

## Files you may write

`src/gfx.rs`, `src/tactical.rs`, `src/main.rs` (only to read the flag),
and the debug flag list plus the `gfx.rs` notes in `docs/CODE_MAP.md`.

## Done looks like

- `cargo build --release` succeeds with no new warnings.
- With no `--fx` flag nothing in the picture or the rules differs from
  before.
- The report gives command lines for: fire at noon, at dusk, on a clear
  night, in night rain and in a duststorm.

## Must not touch

`src/dice.rs`, `src/data.rs`, any rule, any other document.


## Added after the flares job

- Coloured lights wash out to orange-white on this ground. A red light needs a strongly red colour (the hand flare uses 1.0, 0.13, 0.09). Do the same for any coloured light: push the colour further than looks right on paper.
- A light only 0.25 m above the ground skims it and dies within 3 m. Lights that must reach across ground sit at least 0.8 m up.
- The camera brightens any evenly lit night until it looks like a dull day. While a sky flare lights the night the camera is held back 1.5 stops (`SKY_NIGHT_STOPS` in `src/tactical.rs`). Do not change that; do not add another whole-map light without a matching hold-back, and report it if you do.
- The flare constants in `src/tactical.rs` were tuned by the lead by eye. Do not change them.
