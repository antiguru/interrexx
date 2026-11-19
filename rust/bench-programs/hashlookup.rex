/* Hash-store lookup: a million hits and a million misses on a 16-entry and on
   a 100,000-entry Directory, the probe path with a short and a long table. */
small = .directory~new
do i = 1 to 16
    small~put(i, 'k'i)
end
large = .directory~new
do i = 1 to 100000
    large~put(i, 'k'i)
end
n = 1000000
hits = 0
miss = 0
do i = 1 to n
    if small~at('k'||(i // 16 + 1)) \== .nil then hits = hits + 1
    if small~at('x'i) == .nil then miss = miss + 1
    if large~at('k'||(i // 100000 + 1)) \== .nil then hits = hits + 1
    if large~at('x'i) == .nil then miss = miss + 1
end
say hits miss
