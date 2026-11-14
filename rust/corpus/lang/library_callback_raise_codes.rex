/* RaiseException0 through orxmethod with numbers the catalogue has no text
   for: 98.941, naming the code as a whole number where its major has text
   and as major.minor where it has none. The texts and the additional items of
   the errors are not specified (r[conv.error.text], r[conv.error.additional]),
   so the codes are all that is printed. */
t = .T~new
do n over .array~of(99999, 0, 40, 40005, 98941)
  say n '->' try(t, n)
end
exit
try: procedure
  use arg t, n
  signal on syntax
  t~r0(n)
  return 'no raise'
syntax:
  o = condition('o')
  return o~code o~rc
::class T
::method r0 external "LIBRARY orxmethod TestRaiseException0"
