# Handoff: the design fan-out, second round

Written 2026-11-13 by the lead (Fable) for the next session's lead. It
picks up the design fan-out where the first round stopped. Read
`AGENTS.md` first; its rules stand above this file.

## Your job

Run the thirteen design jobs that were not started, judge each result by
eye as the harsh art director in `docs/ART_BIBLE.md`, redo what falls
short, keep the documents in step, and end with a plain report: what was
made, what was rejected and why, what waits on the owner, and how many
command-line runs were made at each model level.

## Where everything is

| What | Where |
|---|---|
| Where things stand | `docs/STATUS.md` (tenth session paragraph, "Next, in order", "Known flaws") |
| The job table and the new proposals (model loader, texture pass, map recipes, destruction) | `docs/ROADMAP.md`, "Design fan-out, first round (2026-11-13)" |
| Open questions for the owner, numbered | `docs/ROADMAP.md`, "Still to decide with the user", items 62 to 69 |
| The owner's answers from six question rounds | `docs/LORE.md`, "Tenth session (2026-11-13): the design fan-out" |
| The briefs, one file per job | `docs/briefs/` (`README.md` there, and `run_job.sh`) |
| Creature verdicts and what to fix next pass | `source_assets/blender/creatures/VERDICTS.md` |
| Each creature's own notes | `source_assets/blender/creatures/<name>/sheet.md` and `rounds.md` |
| Sizes of everything | `source_assets/scale/scale_sheet.json`, helper `scale_lib.py` |
| Lessons about light and models | `docs/ART_BIBLE.md`, "Lessons from the first round (2026-11-13)" |
| Light code, flags and tuned numbers | `docs/CODE_MAP.md` (`gfx.rs in detail`, `tactical.rs in detail`, the debug flag list) |
| The owner's creature pictures | `reference/`, and `docs/lore_refs/` |

## The owner's rules for this work

1. Questions first, through the question tool, recommended option first.
   Do not invent lore, names or rules. Record answers in `docs/LORE.md`
   as they are made.
2. A brief leaves no design choice to the run that receives it. Runs carry
   out; they do not decide. A run that meets an uncovered choice stops and
   reports. The lead then decides, or asks the owner.
3. All sub-agent work goes through the `claude` command line, which is on
   the owner's API credits. Do not use the built-in agent or workflow
   tools for it.
4. Models: `claude-fable-5-2` for design and animation jobs (models,
   textures, motion, pictures, screen layouts); `opus` for rule code and
   file handling; `sonnet` for table rows, wiring to a finished layout,
   documents and checks; `haiku` for commits.
5. Each job writes only inside its own folder. Shared files (`src/`, data
   tables, `docs/NAMES.md`, `docs/CODE_MAP.md`) are changed by one job at
   a time, never two at once.
6. Blender work uses a background Blender per job, never the shared open
   one.
7. No job launches the game. Only the lead does, sparingly, one launch at
   a time, with `DISPLAY=:0`.
8. No job commits or pushes. Commit or push only when the owner says so.
9. At most 4 jobs at once.
10. Judge every result by looking at the pictures. Never call a job done
    on the run's word alone.
11. The owner chose to run the command line with permission checks off.
    Only the brief's wording holds a run back, so check `git status`
    after every job.

## How to run a job

```bash
docs/briefs/run_job.sh G3_fire claude-fable-5-2
```

Run it in the background and wait for it to finish; do not poll. A
creature job takes 10 to 40 minutes and cost $5 to $10 in the first
round. Then:

- Creature or model job: open `renders/contact.png` in its folder.
- Light or effect job: build, then launch the game yourself with the
  commands the run lists, adding `--screenshot <scratch file>`. It exits
  by itself.
- After any job that touched `src/`: `cargo build --release`,
  `cargo test --release`, and the self-test to a scratch file.

## The jobs, in order

Two lanes. The `src/` lane is strictly one at a time. The folder lane can
fill the other slots.

**`src/` lane, one at a time:** `G3_fire` (Fable), `G4_glow` (Fable),
`G5_skies` (Fable), `K1_two_metre_squares` (Opus), `D1_docs_decisions`
(Sonnet).

**Folder lane:** `C09_taker`, `C10_builder`, `R1_ribbons`,
`H1_soldier_kit` (all Fable) and `P1_mpfb_fetch` (Sonnet) can start at
once. Then `H2_head` (after P1), `H3_portraits` (after H2),
`R2_dress_uniform` (after H2, H3, R1 and P1), and last `S1_size_checks`
(Sonnet, after every creature).

Before running, read each brief once. G3, G4, G5 and K1 already carry the
lessons from the flashlight and flare jobs at their end.

## Things that bit the first round

- **Blender is a Flatpak.** Run it as
  `flatpak run org.blender.Blender -b --python "<full path>/script.py"`.
  It cannot read `/tmp` and needs the full path to the script.
- **Night exposure.** Light strengths from a brief burnt white. Divide by
  about 35, then look. Flashlights went from 2.5 to 0.07.
- **Tuned numbers in `src/tactical.rs` are the lead's.** `FLASH_POOL`,
  the `FLARE_*` and `SKY_*` constants and `SKY_NIGHT_STOPS` were set by
  eye. A later job must not change them.
- **`--frametime` reads 0.00 ms in screenshot mode.** Frame cost has to be
  measured another way; it has not been measured for flashlights or
  flares.
- **Runs report honestly but never see the game.** Every light job came
  back "built, not looked at". Expect to tune by eye afterwards; budget
  two to four launches per light job.
- **The self-test writes to its argument.** Point it at a scratch file,
  never at a tracked file in `checks/`.
- **Downloads from Poly Haven** (needed by G5) are blocked on the home
  network unless the Tailscale exit node is on. Use a plain client name;
  send no personal details.

## Owed from the first round

- Two flashlight pictures: a dusk dust storm, and noon with `--lights on`.
- The frame cost of flashlights and of flares.
- Hand flare: it takes all four lamp shadows, so flashlights cast none
  while it burns. Decide whether that is acceptable once fire and glow
  also want shadows.
- The debug sky flare never burns out; burn time is a rule for a later job.
- Second-pass fixes for each creature, listed in `VERDICTS.md`. Do these
  with the texture pass, not before.

## Waiting on the owner

Ask these through the question tool before the job that needs them. The
numbered list is in `docs/ROADMAP.md`, items 62 to 69.

- Before the texture pass: which creatures carry moss, grass or growth.
- Before K1 is called finished: rules for creatures larger than one
  square, and forms that spill past their squares.
- Before the Maul's next pass: whether 3.5 m includes the spike; whole
  spike or stump when dead.
- Before flares become an item: the sky flare's name, cost and numbers.
- Any time: whether a lit soldier is easier to spot; keep or delete the
  old ground square under `--lights off`; whether to schedule map recipes
  and the destruction pipeline as proposed; which props do something when
  destroyed; whether some soldiers spot the Gorger's sign better.

## What the first round cost

Twelve command-line runs, all Fable, about $78.50: the scale sheet, two
light jobs, eight creatures, and one run killed at the start. No Opus,
Sonnet or Haiku runs were made in the design round.
