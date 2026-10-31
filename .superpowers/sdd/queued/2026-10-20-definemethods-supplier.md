**OPEN: accepted for the Phase 9d close by Moritz 2026-10-31 ("fix now.").** `Class~defineMethods` does not send a collection subclass's `supplier`.

# `defineMethods` and a collection subclass's `supplier`

Found 2026-10-20 (`task-6.12-rereview.md`, queue candidate); it predates Task 6.12. Given a
collection subclass, the oracle's `defineMethods` sends the collection's `supplier`, and sends made
from that `supplier` reach a security manager (`METHOD DEFINE INNER,return 2`). The crate does not
send `supplier`.
