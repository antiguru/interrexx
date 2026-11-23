/* Class~define repeatedly replaces one method while some instances stay
   live. Existing instances see the replacement. Allocating between defines
   exercises collection of methods that are no longer named or running. */
n = 3000
keep = .array~new(50)
do i = 1 to n
  .k~define('M', 'return' i)
  keep[i // 50 + 1] = .k~new
  do j = 1 to 20
    x = .array~of(i, j, .directory~new)
  end
end
say keep[1]~m keep[50]~m
::class k
