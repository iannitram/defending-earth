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
# Job G1: moving lights in the battle view, and weapon flashlights

## Goal

Give the battle view lights other than the sun and sky, then use them for
the soldiers' weapon flashlights. Later jobs (flares, fire, the Glow)
reuse the same light system, so build it for them too.

## The settled design

**The light system (in `src/gfx.rs`).**
- A small light record: position, direction, colour and strength (how
  bright a white surface is at 1 m, on the code's existing brightness
  scale), reach in metres, and two cone angles. A point light is a light
  with no cone.
- The scene shader takes up to 24 such lights per frame. Brightness
  falls with the square of distance and is faded smoothly to zero at the
  light's reach, so a light never ends in a hard edge. Light on a
  surface depends on the angle it strikes at. On wet ground (rain) it
  also gives a soft highlight, using the wetness the shader already has.
- The battle code hands over the lights each frame. If there are more
  than 24, keep the ones nearest the camera's point of interest.
- **Shadows from up to 4 of these lights.** The four cone lights nearest
  the camera's point of interest cast shadows, using four 1024 by 1024
  depth pictures packed into one 2048 by 2048 texture, drawn the same
  way the sun's shadow is drawn now. Soft edge of about two texels.
  Build the system without shadows first and make sure it builds; then
  add the shadows. If shadows cannot be made to work in macroquad, or
  the code shows they must cost more than about 3 ms a frame, leave
  them out, keep the rest, and say so in the report.
- Lights add to the picture before exposure, so the existing automatic
  exposure, bloom and lens effects treat them like everything else.

**The flashlight.**
- One on each living soldier of the player's side. It is fixed under the
  weapon's barrel and points where the soldier faces, tilted 4 degrees
  down, from 1.2 m above the soldier's feet and 0.3 m in front.
- Cone: full brightness inside 9 degrees from the centre line, fading
  smoothly to nothing at 20 degrees. Reach 32 m. Colour a slightly cool
  white (red 0.92, green 0.96, blue 1.0). Strength chosen so that the
  centre of the pool of light on flat ground 10 m ahead is about 2.5 on
  the code's brightness scale.
- **It comes on by itself in the dark.** On when the sun is more than 2
  degrees below the horizon, or when the battle's light level at the
  soldier is as dark as that (heavy storm at dusk). Soldiers do not all
  switch on in the same frame: each waits between 3 and 9 frames,
  different for each soldier, fixed by the soldier's number. There is no
  indoor case to handle yet: the game has no interiors.
- **The beam is seen as a cone only in rain, snow, fog, dust or smoke.**
  In clear air only the pool of light on the ground and on things is
  seen. In weather, draw the beam as a faint soft cone in the air,
  brightest near the lamp and fading along its length, its strength
  following how thick the weather is. It must look like light catching
  drops or dust, not like a solid glowing shape: no hard edge, no flat
  colour.
- The lamp itself, seen from the front, is a tiny point that blows out
  to white. No star, no streaks.
- Enemies carry no flashlights.

**Debug switch.** `--lights auto|on|off`. `auto` is the default and
follows the rule above. `on` forces every flashlight on at any hour.
`off` forces them off.

## Files you may write

`src/gfx.rs`, `src/tactical.rs`, `src/main.rs` (only to read the new
flag), and the debug flag list plus the `gfx.rs` notes in
`docs/CODE_MAP.md`.

## Done looks like

- `cargo build --release` succeeds with no new warnings.
- By reading the code you can show that with `--lights off` nothing in
  the picture or the rules differs from before.
- The report gives command lines for: a clear night, a rainy night, a
  duststorm at dusk, and noon with `--lights on`.
- The report says whether shadows from these lights are in, and what the
  system costs as far as can be told without running it.

## Must not touch

`src/dice.rs`, `src/data.rs`, any rule, any other document.
