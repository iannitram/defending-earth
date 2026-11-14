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
# Job G2: the hand flare and the sky flare (the look only)

## Goal

Draw the two flares in the battle view, using the moving-light system
that already exists in `src/gfx.rs` (read it first). This job is the
look only. Flares as items a soldier carries and throws are a rule, and
that is a later job: do not add items, costs or actions.

## The settled design

**The hand flare.** Thrown like a grenade, it burns where it lands for
several turns.
- *Light:* a point light 0.25 m above the ground. Colour a hard red
  (red 1.0, green 0.36, blue 0.28). Reach 14 m. Strength chosen so that
  flat ground 2 m away is about 8 on the code's brightness scale and
  ground 12 m away (six squares) is still just lit, about 0.2.
- *Core:* a spot about 6 cm across that is white-hot (about 40 on the
  scale), so it blows out to white with a red-pink edge from the bloom
  that already exists. The ground within 0.4 m also blows out.
- *Flicker:* brightness times (0.85 plus 0.15 times a noise that changes
  about 11 times a second), on top of a slower swell of 5 percent about
  twice a second. Never a regular pulse.
- *Sparks:* 6 to 10 at a time, tiny white-red points thrown up to 0.3 m
  and gone within a third of a second.
- *Smoke:* a thin pale column that leans with the wind, lit red near the
  flare and fading to nothing by 4 m up. Soft, not puffy balls.
- *Shadows:* things near it throw long shadows away from it. Use four
  sideways shadow views round the nearest hand flare, sharing the
  shadow texture the moving lights already use. If that system is not
  there, or it cannot be done inside the budget, leave the shadows out
  and say so.
- *Ground:* a small scorched spot under it.

**The sky flare.** Fired upward, it hangs under a small parachute and
lights most of the map for a few turns.
- *Light:* starts 70 m above the map and sinks 1.2 m a second, swinging
  2 m from side to side about every 5 seconds and drifting with the
  wind. Colour a warm white with a little red (red 1.0, green 0.86,
  blue 0.78). It lights the whole map. Strength chosen so that open
  ground under it is about one eighth as bright as the same ground on
  an overcast day.
- *Shadows:* when the sun is below the horizon, the sun's own shadow
  picture is not in use. Drive it from the sky flare instead, so the
  whole map gets hard shadows that swing slowly as the flare swings.
  With the sun up, the sky flare casts no shadows.
- *Core:* a white-hot point that blows out to white. Above it a tiny
  dark parachute shape about 1 m across and a thin smoke trail lit from
  below.
- *Flicker:* 8 percent, changing 6 to 9 times a second.

**Debug switches.** `--fx` takes a list separated by commas. This job
adds `flare` (one hand flare burning on the ground four squares from the
middle of the map, toward the camera) and `skyflare` (one sky flare
above the middle of the map, at 45 m, already burning). If `--fx`
already exists, add to it; if not, create it so later jobs can add
`fire` and `glow`.

## Files you may write

`src/gfx.rs`, `src/tactical.rs`, `src/main.rs` (only to read the flag),
and the debug flag list plus the `gfx.rs` notes in `docs/CODE_MAP.md`.

## Done looks like

- `cargo build --release` succeeds with no new warnings.
- With no `--fx` flag nothing in the picture or the rules differs from
  before.
- The report gives command lines for: a hand flare on a clear night, a
  hand flare in night rain, a sky flare on a clear night, both together,
  and a hand flare at noon (where it should look weak).

## Must not touch

`src/dice.rs`, `src/data.rs`, any rule, any item table, any other
document.
