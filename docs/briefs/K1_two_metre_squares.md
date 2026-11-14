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
# Job K1: draw the battlefield with 2 m squares

## Goal

The owner has decided that a battlefield square is 2 m across. The code
draws it at 1.65 m (`TILE` in `src/tactical.rs`). Change what is drawn
to 2 m, without changing the result of any rule.

## The settled design

- The one sheet of sizes is `source_assets/scale/scale_sheet.json`
  (`anchor.square_m` is 2.0, `anchor.soldier_height_m` is 1.8). Do not
  change it.
- **Drawing uses 2 m.** `TILE` becomes 2.0. Everything placed or sized
  from it (the ground, props, the camera's framing, markers) follows.
- **Rules keep their results exactly.** Read every use of `TILE` in
  `src/`. Wherever it feeds a rule (anything that ends in a dice roll,
  damage, sight, range, movement cost or a stray shot's path, such as
  the stray-shot spread near line 607 of `src/tactical.rs`), keep the
  old number by using a new constant `RULE_SQUARE_M: f32 = 1.65`, with a
  comment saying that rule distances are moved to 2 m in a later job.
  List every place you changed and which constant it now uses.
- **A soldier is drawn 1.8 m tall.** Find where the soldier figure's
  height is set and make it 1.8 m; report the old value. Other figures
  and props keep their sizes in metres.
- Add a unit test that reads `source_assets/scale/scale_sheet.json` as
  text and fails if `TILE` is not the sheet's `square_m`. Use only what
  the project already depends on (look in `Cargo.toml`); add no new
  dependency. If the project has no way to read JSON, find the number
  by plain text search in the test.
- **Proof.** Before changing anything, build and run
  `cargo test --release` and keep the output. After the change, build
  and run it again. You may not launch the game, so the scripted battle
  self-test is run by the lead afterwards. Your proof is the tests plus
  a read-through showing no rule value changed.

## Files you may write

`src/tactical.rs`, other files in `src/` only where they use `TILE`, and
the lines about the square size in `docs/CODE_MAP.md`.

## Done looks like

`cargo build --release` and `cargo test --release` pass. The report
lists every use of `TILE`, whether it is drawing or rule, and what it
uses now. If any use cannot be sorted clearly into drawing or rule,
stop with `UNDECIDED:` and list them, changing nothing.

## Must not touch

`src/dice.rs`, `src/data.rs`, the scale sheet, any file in `checks/`,
any other document.


## Added after the flares job

- Coloured lights wash out to orange-white on this ground. A red light needs a strongly red colour (the hand flare uses 1.0, 0.13, 0.09). Do the same for any coloured light: push the colour further than looks right on paper.
- A light only 0.25 m above the ground skims it and dies within 3 m. Lights that must reach across ground sit at least 0.8 m up.
- The camera brightens any evenly lit night until it looks like a dull day. While a sky flare lights the night the camera is held back 1.5 stops (`SKY_NIGHT_STOPS` in `src/tactical.rs`). Do not change that; do not add another whole-map light without a matching hold-back, and report it if you do.
- The flare constants in `src/tactical.rs` were tuned by the lead by eye. Do not change them.
