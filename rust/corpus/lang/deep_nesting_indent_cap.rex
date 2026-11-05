/* The clause echo on an error report saturates at 40 columns of indent.

   Nesting depth drives the `*-*` echo's indent at two spaces per level, and
   that growth STOPS at 40. Measured against the oracle with exactly this
   shape -- plain DO blocks around one failing clause, no call and no
   INTERPRET anywhere -- depth 18 echoes at 36, depth 19 at 38, depth 20 at
   40, and depths 21, 25 and 30 all echo at 40 as well.

   The interpreter once printed 50 here where the oracle prints 40, a
   divergence that needs nothing beyond plain DO blocks to show.





   The depth is 25 rather than 21 on purpose. At 21 an implementation that
   subtracted a level, or that clamped one bump too late, would still land on
   40 by accident; at 25 the uncapped answer is 50 and only a real clamp
   reaches 40.

   The two SAY lines are there so the nest is entered rather than merely
   parsed: an implementation that skipped the bodies entirely would still
   raise 42.3 from the last one, and stdout is what tells the two apart. */

say 'entering'
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
do
say 'at depth 25'
say 1/0
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
end
