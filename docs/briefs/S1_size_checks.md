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
# Job S1: the automatic size checks

## Goal

Write the script that checks every finished model against the scale sheet.

## The settled design

- The sheet is `source_assets/scale/scale_sheet.json`. Read it; do not
  change it. Squares are `anchor.square_m` metres across.
- Each model job writes a `size.json` beside its model, somewhere under
  `source_assets/blender/`, in this form: `{"name", "width_m", "depth_m",
  "height_m", "footprint_squares": [w, d], "triangles"}`. A folder may
  hold several, named `size_<form>.json`, for a thing with several forms.
- Write `source_assets/scale/check_sizes.py` in plain Python 3 with only
  the standard library (no Blender). It finds every `size*.json` under
  `source_assets/blender/` and checks, for each:
  1. The name is in the sheet's `things`.
  2. `footprint_squares` equals the sheet's footprint for that thing (or
     one of its other declared footprints, such as `footprint_head_out`).
  3. Width is at most footprint width times the square size, and depth at
     most footprint depth times the square size. A thing listed in the
     sheet's `cheats` by name may overhang; report it as "allowed
     overhang" instead of a failure.
  4. Where the sheet declares `height_m`, the measured height is within 5
     percent of it.
  5. Triangles are at most 40,000.
- It also checks the sheet itself: a soldier with kit
  (`soldier_with_kit_width_m`, `soldier_height_m`) fits through the door
  (`door_width_m`, `door_height_m`), low cover is lower than the soldier
  and full cover is taller.
- Output: a plain table printed to the screen, and the same table written
  to `source_assets/scale/size_report.md`. Exit code 0 when everything
  passes, 1 when anything fails. Things in the sheet with no `size.json`
  yet are listed as "not modelled yet" and do not fail.

## Files you may write

`source_assets/scale/check_sizes.py` and
`source_assets/scale/size_report.md`. Nothing else.

## Done looks like

`python3 source_assets/scale/check_sizes.py` runs from the project folder,
prints the table and writes the report. Test it with a made-up
`size.json` in a temporary folder outside the project to prove that a
model that is too wide fails; remove the temporary folder afterwards.

## Must not touch

The scale sheet, any model folder, any game code.
