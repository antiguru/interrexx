/* List operations at both ends and in the middle of a long list: append,
   remove at the front and the back, insert at the front and after a handle
   in the middle, and walks by next and previous.  Each is constant time per
   operation in a linked list, so the run is linear in the operation count
   whatever the length of the list. */
l = .list~new
do i = 1 to 100000; l~append(i); end
sum = 0
n = 24
do pass = 1 to n
  /* rotate: take from the front, append at the back */
  do 50000; l~append(l~remove(l~first)); end
  /* the other way: take from the back, insert at the front */
  do 50000; l~insert(l~remove(l~last), .nil); end
  /* insert after a handle in the middle and remove it again */
  mid = l~first
  do 1000; mid = l~next(mid); end
  do k = 1 to 20000; h = l~insert(k, mid); l~remove(h); end
  /* walk forward and back */
  h = l~first
  do while h \= .nil; sum = sum + 1; h = l~next(h); end
  h = l~last
  do while h \= .nil; sum = sum + 1; h = l~previous(h); end
end
say l~items l~firstItem l~lastItem sum
