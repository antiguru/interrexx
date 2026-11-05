/* A mutation-testing witness: TO, BY and FOR are evaluated in the
   order they were WRITTEN (`Controlled::order`, recorded because the
   expressions can have side effects), never a fixed TO-then-BY-then-FOR
   order. Here there is no way to make evaluation order
   observable through a genuine side effect -- there are no function calls
   or message sends, so TO/BY/FOR can only ever be literals or variable
   reads -- but TRACE's own `>K>` lines are themselves an evaluation-order
   effect: they print in the order each keyword is actually evaluated, once,
   at loop setup, regardless of what value each keyword holds.

   `LEAVE` on the first pass keeps this program to exactly one loop pass, so a
   divergence here is attributable to the order defect it targets and to
   nothing else. A Controlled loop's re-tested pass is witnessed by
   tests/trace_oracle/controlled_loop.rex.




   Measured: the oracle prints `>K>` for TO, then BY, then FOR, in that
   written order. Mutating the evaluation loop to iterate `ctrl.order` in
   reverse flips the three lines to FOR, BY, TO -- the values on each line
   are unchanged either way, only their sequence differs. */
trace r
do i = 1 to 2 by 1 for 5
  say i
  leave
end
