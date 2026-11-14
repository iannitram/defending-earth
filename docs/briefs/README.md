# Design job briefs

The briefs for the design fan-out of 2026-11-13. Each file is one whole
job: it leaves no design choice to the run that receives it. If a run
meets a choice the brief does not cover, it stops and reports back; it
does not guess.

Read `docs/HANDOFF.md` first. It says which of these have run, which have
not, the order, and the rules.

- Done: `S0`, `G1`, `G2`, `C01` to `C08`. Kept for the record.
- Not run: `G3`, `G4`, `G5`, `K1`, `D1`, `C09`, `C10`, `R1`, `P1`, `H1`,
  `H2`, `H3`, `R2`, `S1`.

`run_job.sh <job> <model>` runs one brief from the project folder and
prints the result, the cost and `git status`. Results go to a scratch
folder named by `RESULTS` (default `/tmp/de_results`), never into the
project.

The briefs name the project by its full path on the owner's machine.
Change it if the project moves.
