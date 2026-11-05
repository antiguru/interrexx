# interrexx

Read `docs/developing.md` before working here; design notes are in `docs/design/`.

* Run ooRexx and interrexx only under a memory cap (`systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 ...`).
* Never modify `third_party/`, `ootest/` or `oodocs/`, and keep to the clean-room rule in `docs/developing.md`.
