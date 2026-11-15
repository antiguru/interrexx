/* Hash-store duplicate keys: a Relation with 20,000 items under one index and
   20,000 other indexes with one item each, read by ALLAT and AT and drained
   by REMOVE. */
r = .relation~new
do i = 1 to 20000
    r~put(i, 'dup')
    r~put(i, 'other'i)
end
total = 0
n = 1500
do n
    total = total + r~allAt('dup')~items
    total = total + r~at('dup')~length
end
do i = 1 to 20000
    total = total + r~remove('dup')~length
end
say total r~items
