# interrexx

Read `docs/developing.md` before working here; design notes are in `docs/design/`. The standing decisions and their reasons are in `docs/design/decisions.md`: read it before proposing to change one.

* Run ooRexx and interrexx only under a memory cap (`systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 ...`; on macOS, `scripts/rss-cap.sh 8388608 ...`).
* Never modify `third_party/`, `ootest/` or `oodocs/`, and keep to the clean-room rule in `docs/developing.md`.
* Run a claim that a case is unreachable, or that a rule holds for all or none of a set, before stating it: derive it or test it, don't infer it from reading code.
* A test adds coverage only if it fails without the change it covers: check that by reverting the change (mutation runs need `--no-fail-fast`), and prefer asserting a side effect over a return value a deleted line would not change.
* An expected failure, skip or "fails upstream too" entry needs its stated cause checked in the setup a real user has: a failure both engines share can come from the shared harness, not from ooRexx.
