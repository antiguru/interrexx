t = .methodtester~new
do size over .array~of(2305843009213693953, 2305843009213693952, 18446744073709551615, 5)
  say size api(t, size)
end
::requires 'METHODPackage.cls'
::routine api
  use arg t, size
  signal on syntax
  numeric digits 20
  return 'ok' t~TestNewArray(size)~size
syntax: return condition('o')~code condition('o')~message
