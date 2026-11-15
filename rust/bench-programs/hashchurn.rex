/* Hash-store churn: a million put/remove cycles on a Table that never holds
   more than eight entries, so removals dominate and the store must reuse the
   space they free. */
numeric digits 18
t = .table~new
n = 1000000
total = 0
do i = 1 to n
    if i > 8 then total = total + t~remove(i - 8)
    t~put(i, i)
end
say total t~items
