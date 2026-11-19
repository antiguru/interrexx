/* Hash-store copy: 6,000 COPY calls of a 10,000-entry Table. */
t = .table~new
do i = 1 to 10000
    t~put(i, 'k'i)
end
total = 0
n = 6000
do n
    total = total + t~copy~items
end
say total
