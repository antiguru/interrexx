/* Hash-store iteration: 200 passes of ALLINDEXES, SUPPLIER and DO OVER over a
   10,000-entry StringTable. */
s = .stringtable~new
do i = 1 to 10000
    s[i] = i
end
total = 0
n = 200
do n
    total = total + s~allIndexes~items
    sup = s~supplier
    do while sup~available
        total = total + sup~item
        sup~next
    end
    do idx over s
        total = total + 1
    end
end
say total
