parse arg instance
n = (instance = 'B') + 1
call Lend n
if instance = 'B' then do
  say 'B, its own kept handle:' UseKept(2)
  say "B, A's kept handle:" UseKept(1)
end
::requires 'keptacross' LIBRARY
