/* PUSH and QUEUE, under TRACE R throughout.                                 */
/*                                                                           */
/* WHAT THIS PROGRAM CAN AND CANNOT PIN. Nothing that reads the queue back   */
/* -- PULL, PARSE PULL, QUEUED() -- is used here, so this program cannot     */
/* differentially witness which end of the queue a line landed on or in what */
/* order. What it CAN pin, and what TRACE R is here for: PUSH/QUEUE evaluate */
/* their own expression, render it to string form -- a string literal, a     */
/* concatenation, and a number (`queue 42`: a number renders through a       */
/* different path than a literal does) -- and trace the result exactly like  */
/* SAY does (the oracle's own RexxInstructionQueue::execute shares SAY's     */
/* evaluateStringExpression). A bare PUSH/QUEUE with no expression traces a  */
/* null string rather than being skipped or erroring. Without TRACE R this   */
/* program would produce empty stdout, empty stderr, rc 0 whether PUSH/QUEUE */
/* store anything at all -- a test that cannot fail -- so the >>> lines below*/
/* are the whole point, not decoration. The interleaved storage order, and   */
/* whether run.rs's arms write to the queue at all, are pinned by queue.rs's */
/* own unit tests instead (rust/crates/interrexx/src/queue.rs).              */
/* lang/pull_queue.rex reads the order back on stdout, and                   */
/* crates/interrexx/tests/input_oracle.rs's `queue-round-trip` row does it   */
/* against a console holding different lines.                                */
/*                                                                           */
/* Ends with a bare EXIT, whose value line is pinned against the oracle by   */
/* tests/trace_oracle/exit_value.rex.                                        */
/*                                                                           */
/*                                                                           */
/*                                                                           */
/*                                                                           */
/*                                                                           */
/*                                                                           */

trace r
a = 'hello'
push a
queue 'world'
push a 'there'
queue 42
queue
push
exit
