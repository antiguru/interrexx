/* A notifier that fails while a held send's failure is unwinding replaces
   that failure, above the levels it was raised in; the message keeps its
   own condition. */
m = .message~new(.t~new, 'boom')
m~notify(.bad~new)
signal on syntax
m~send
say 'not reached'
syntax:
  c = condition('o')
  say 'trapped' c~code c~position m~hasError m~completed m~errorCondition~code
  do l over c~traceback -- numbered levels only: a built-in one may be absent, r[class.stackframe.builtin]
    if l~word(1)~datatype('W') then say ' ' l
  end
::class t
::method boom
  return 1/0
::class bad inherit MessageNotification
::method messageComplete
  use arg msg
  say 'notifier sees' msg~errorCondition~code
  return .nil~foo
