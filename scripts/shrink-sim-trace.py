#!/usr/bin/env python3
"""Minimize a reproducible failing interrexx simulation trace."""

import argparse
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import tempfile


SIM_REPORT = re.compile(
    rb"rexx-sim: seed=\d+ policy=\S+ steps=\d+ contended=\d+ switches=\d+"
    rb"(?: trace=[0-9a-f]{16})?(?: hash=\d+)? profile=(?:debug|release) stack=\d+\n\Z"
)


def trial_dir_path(path):
    if "," in str(path):
        raise ValueError(f"simulation paths cannot contain commas: {path}")
    return path


def run(command, mode, timeout):
    env = os.environ | {"REXX_SWITCH_MODE": mode}
    with subprocess.Popen(
        command,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
        start_new_session=True,
    ) as process:
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
            return None
    if "REXX_SIM_CHILD" not in env:
        stderr = SIM_REPORT.sub(b"", stderr)
    return process.returncode, stdout, stderr


def shrink(args, directory):
    command = [str(args.rexx), *args.program]
    original = trial_dir_path(args.trace.resolve())
    canonical = trial_dir_path(directory / "canonical.trace")
    trial = directory / "trial.trace"
    omitted = directory / "omit"

    def replay(source, output, omit=None):
        output.unlink(missing_ok=True)
        mode = f"sim:replay={source},trace={output}"
        if omit is not None:
            mode += f",omit={omit}"
        signature = run(command, mode, args.timeout)
        return signature, output.read_bytes() if output.exists() else None

    decisions = original.read_bytes().splitlines()[1:]
    baseline, best = replay(original, canonical)
    if baseline is None or best is None or baseline[0] <= 0:
        raise ValueError("the input trace did not replay to a completed failure")
    if best.splitlines()[1:] != decisions:
        raise ValueError("strict replay did not consume the input trace")
    if replay(canonical, trial) != (baseline, best):
        raise ValueError("the input trace's failure did not repeat under strict replay")

    remaining = list(range(len(decisions)))
    removed = set()
    partitions = 2
    attempts = 0
    while remaining and attempts < args.trials:
        width = (len(remaining) + partitions - 1) // partitions
        for start in range(0, len(remaining), width):
            if attempts == args.trials:
                break
            attempts += 1
            block = remaining[start : start + width]
            subset = removed.union(block)
            omitted.write_text("".join(f"{index}\n" for index in sorted(subset)))
            signature, recorded = replay(original, trial, omitted)
            if signature != baseline or recorded is None:
                continue
            if replay(trial, canonical) != (baseline, recorded):
                continue
            if len(recorded.splitlines()) <= len(best.splitlines()):
                best = recorded
            removed = subset
            del remaining[start : start + width]
            partitions = max(2, partitions - 1)
            break
        else:
            if width == 1:
                break
            partitions = min(len(remaining), partitions * 2)

    return best, len(decisions), attempts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rexx", type=Path, default=Path("rust/target/debug/rexx"))
    parser.add_argument("--timeout", type=float, default=10, help="seconds per replay")
    parser.add_argument("--trials", type=int, default=256, help="maximum omission trials")
    parser.add_argument("trace", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("program", nargs=argparse.REMAINDER, help="the original rexx arguments")
    args = parser.parse_args()
    if not args.program:
        parser.error("supply the original rexx program and its arguments")
    if not math.isfinite(args.timeout) or args.timeout <= 0 or args.trials <= 0:
        parser.error("timeout and trials must be positive")
    if args.output.exists():
        parser.error(f"output already exists: {args.output}")
    try:
        with tempfile.TemporaryDirectory(prefix="rexx-sim-shrink-") as directory:
            trace, original, attempts = shrink(args, Path(directory))
        with args.output.open("xb") as output:
            output.write(trace)
    except (OSError, ValueError) as error:
        parser.exit(1, f"{parser.prog}: {error}\n")
    print(f"{original} -> {len(trace.splitlines()) - 1} recorded decisions in {attempts} trials")


if __name__ == "__main__":
    main()
