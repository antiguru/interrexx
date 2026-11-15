/* Hash-store churn: a million put/remove cycles on a Table that never holds
   more than eight entries, so removals dominate and the store must reuse the
   space they free. */
t = .table~new
n = 1000000
total = 0
do i = 1 to n
    t~put(i, i)
    if i > 8 then total = total + t~remove(i - 8)
end
say total t~items
