---
name: experiment
model: opus
description: Run a hypothesis-driven experiment and record evidence.
user-invocable: true
allowed-tools: Read, Write, Edit, Bash, Grep, Glob
---
# Experiment

Run a hypothesis-driven experiment.

## Steps

1. Capture a falsifiable hypothesis.
2. Capture the falsification criterion.
3. Capture the method.
4. Assign the next `EXP-<NNN>` from `kb/Experiments/LOG.md`.
5. Create `kb/Experiments/EXP-<NNN>.md`.
6. Run the method and record evidence.
7. Mark the outcome as confirmed, falsified, or inconclusive.
8. Append the result to `kb/Experiments/LOG.md`.

## Rules

- Do not run an experiment without a falsification criterion.
- Record evidence even if the result is obvious.
- Keep experiment files self-contained.
- Do not rewrite past outcomes; corrections are new experiments.
