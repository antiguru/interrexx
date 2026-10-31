**CLOSED: Implemented at `ad0741664` for the Phase 9d close on 2026-10-31.** Moritz authorized the correction with "fix now.". Collection subclasses now receive `SUPPLIER` through normal dispatch, including protected sends and recursive installation inside that method. Fourteen paired witnesses match on stdout, stderr and exit status, with focused ordinary and forced-collection regressions passing. The evidence and retained native-directory boundary are recorded in `../2026-10-18-phase-9d/task-10-corrections-report.md`.

# `defineMethods` and a collection subclass's `supplier`

Found 2026-10-20 (`task-6.12-rereview.md`, queue candidate); it predates Task 6.12. Given a
collection subclass, the oracle's `defineMethods` sends the collection's `supplier`, and sends made
from that `supplier` reach a security manager (`METHOD DEFINE INNER,return 2`). The crate does not
send `supplier`.
