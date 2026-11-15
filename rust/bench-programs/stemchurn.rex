/* Stem tails: 50,000 tails, 4,000,000 DROP and re-assign cycles over them,
   then one DO OVER. */
numeric digits 18
a. = 0
do i = 1 to 50000
    a.i = i
end
n = 4000000
do i = 1 to n
    k = i // 50000 + 1
    drop a.k
    a.k = i
end
total = 0
do idx over a.
    total = total + a.idx
end
say total
