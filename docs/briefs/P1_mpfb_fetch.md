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
# Job P1: fetch the free human base mesh and face shapes

## Goal

Download the CC0 human base mesh and shape files from the open project
MPFB2 into the project's source assets, with a record of where they came
from. Later jobs build the soldier's body and head from them. The owner
has agreed to this download.

## The settled design

- Source: `https://github.com/makehumancommunity/mpfb2`, release tag
  `v2.0.18`. Use `curl` with the user agent `asset-fetch`. Send nothing
  about the user or this project in any request.
- Fetch only data, never code: from `src/mpfb/data/` at that tag take
  `3dobjs/` (the base mesh), `targets/` (the shape files) and
  `mesh_metadata/`. Also take the files in the repository that state the
  licence of the data (look for `LICENSE*` and any licence file inside
  `src/mpfb/data/`). The simplest way is to download the tag's source
  archive into a new empty folder under `/tmp`, copy out those folders,
  and delete the rest.
- **Check the licence before keeping anything.** The data must be stated
  as CC0 in the files you fetched. If the licence text does not say the
  mesh and targets are CC0, stop with `UNDECIDED:` and quote what it says.
- Do not fetch or keep any Python code from that project (it is GPL and
  must not enter this repository). Do not run anything you downloaded.
- Put the files under `source_assets/mpfb2_cc0/` keeping their folder
  names. Write `source_assets/mpfb2_cc0/SOURCE.md`: what this is, the
  address, the tag, the date (2026-11-13), the licence with the exact
  sentence that states it, and the list of top-level folders kept with
  their size.
- Add one entry to `CREDITS.md` under a new heading `## Human base mesh`:
  MPFB2 / MakeHuman community base mesh and shape targets, CC0, used to
  build soldier bodies and heads; the add-on's code is not used or
  shipped.

## Files you may write

Everything under `source_assets/mpfb2_cc0/`, and the one new section in
`CREDITS.md`.

## Done looks like

The folders are in place, `SOURCE.md` and the credit are written, no `.py`
file is anywhere under `source_assets/mpfb2_cc0/`, and your report gives
the number of shape files and the total size on disk.

## Must not touch

Anything else. Do not install anything into Blender.
