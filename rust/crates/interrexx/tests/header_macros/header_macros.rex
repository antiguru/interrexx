/* The program header_macros.rs runs against each build of macros.c. The
   driver writes the library's name in place of @LIB@, and 1 or 0 in place of
   @CXX@ for whether the library was compiled as C++. */

-- Every arity of a routine and of a method.
say 'routines' hmsum0() hmsum1(1) hmsum2(1, 2) hmsum3(1, 2, 3) hmsum4(1, 2, 3, 4) -
  hmsum5(1, 2, 3, 4, 5) hmsum6(1, 2, 3, 4, 5, 6) hmsum7(1, 2, 3, 4, 5, 6, 7)
say 'routines' hmsum8(1, 2, 3, 4, 5, 6, 7, 8) hmsum9(1, 2, 3, 4, 5, 6, 7, 8, 9) -
  hmsum10(1, 2, 3, 4, 5, 6, 7, 8, 9, 0) hmrow(4, 2)
o = .hm~new
say 'methods' o~msum0 o~msum1(1) o~msum2(1, 2) o~msum3(1, 2, 3) o~msum4(1, 2, 3, 4) -
  o~msum5(1, 2, 3, 4, 5) o~msum6(1, 2, 3, 4, 5, 6) o~msum7(1, 2, 3, 4, 5, 6, 7)
say 'methods' o~msum8(1, 2, 3, 4, 5, 6, 7, 8) o~msum9(1, 2, 3, 4, 5, 6, 7, 8, 9) -
  o~msum10(1, 2, 3, 4, 5, 6, 7, 8, 9, 0) o~mrow(4, 2)

-- Every value code as a parameter and a result.
say 'signed' hmecho_int(-7) hmecho_wholenumber_t(-7) hmecho_int8_t(-7) -
  hmecho_int16_t(-7) hmecho_int32_t(-7) hmecho_int64_t(-7) hmecho___int32_t(-7) -
  hmecho___int64_t(-7) hmecho_intptr_t(-7) hmecho_ssize_t(-7)
say 'unsigned' hmecho_stringsize_t(7) hmecho_uint8_t(7) hmecho_uint16_t(7) -
  hmecho_uint32_t(7) hmecho_uint64_t(7) hmecho___uint32_t(7) hmecho___uint64_t(7) -
  hmecho_uintptr_t(7) hmecho___uintptr_t(7) hmecho_size_t(7) hmecho_logical_t(1)
say 'restricted' hmecho_positive_wholenumber_t(7) hmecho_nonnegative_wholenumber_t(0)
say 'floating' hmecho_double(2.5) hmecho_float(2.5)
say 'strings' hmecho_CSTRING('abc') hmecho_RexxStringObject('def')
a = .array~of(1, 2, 3)
s. = 'default'
m = .mutablebuffer~new('mb')
x = 'referenced'
r = >x
say 'objects' (hmecho_RexxObjectPtr(a) == a) hmecho_RexxArrayObject(a)~items -
  (hmecho_RexxStemObject(s.)~identityHash = s.~identityHash) -
  hmecho_RexxClassObject(.array)~id -
  (hmecho_RexxMutableBufferObject(m)~identityHash = m~identityHash) -
  hmecho_RexxVariableReferenceObject(r)~value
say 'pointers' hmismarker(hmpointer()) hmismarker(hmecho_POINTER(hmpointer())) -
  hmismarkerstring(hmpointerstring()) -
  hmismarkerstring(hmecho_POINTERSTRING(hmpointerstring()))

-- The optional forms, omitted and given.
say 'omitted' hmopt_int() hmopt_wholenumber_t() hmopt_double() hmopt_stringsize_t() -
  hmopt_float() hmopt_int8_t() hmopt_int16_t() hmopt_int32_t() hmopt_int64_t() -
  hmopt_uint8_t() hmopt_uint16_t() hmopt_uint32_t() hmopt_uint64_t() hmopt_intptr_t() -
  hmopt_uintptr_t() hmopt_logical_t() hmopt_size_t() hmopt_ssize_t() -
  hmopt_positive_wholenumber_t() hmopt_nonnegative_wholenumber_t()
say 'given' hmopt_int(3) hmopt_wholenumber_t(3) hmopt_double(3) hmopt_stringsize_t(3) -
  hmopt_float(3) hmopt_int8_t(3) hmopt_int16_t(3) hmopt_int32_t(3) hmopt_int64_t(3) -
  hmopt_uint8_t(3) hmopt_uint16_t(3) hmopt_uint32_t(3) hmopt_uint64_t(3) hmopt_intptr_t(3) -
  hmopt_uintptr_t(3) hmopt_logical_t(1) hmopt_size_t(3) hmopt_ssize_t(3) -
  hmopt_positive_wholenumber_t(3) hmopt_nonnegative_wholenumber_t(3)
say 'null' hmoptnull_RexxObjectPtr() hmoptnull_CSTRING() hmoptnull_POINTER() -
  hmoptnull_RexxStringObject() hmoptnull_RexxArrayObject() hmoptnull_RexxStemObject() -
  hmoptnull_POINTERSTRING() hmoptnull_RexxClassObject() -
  hmoptnull_RexxMutableBufferObject()
say 'not null' hmoptnull_RexxObjectPtr(a) hmoptnull_CSTRING('c') -
  hmoptnull_POINTER(hmpointer()) hmoptnull_RexxStringObject('s') -
  hmoptnull_RexxArrayObject(a) hmoptnull_RexxStemObject(s.) -
  hmoptnull_POINTERSTRING(hmpointerstring()) hmoptnull_RexxClassObject(.array) -
  hmoptnull_RexxMutableBufferObject(m)

-- Ten parameters of ten codes.
say 'mixed' hmmixed(-8, 16, -32, 64, 0.5, 'str', 1, 9, -9, 99)

-- The special arguments.
say 'name' hmname() 'HmName'()
args = hmargs(1, , 3)
say 'arglist' args~size args~items args[1] args[3]
say 'self' (o~self == o) 'super' o~super~id 'scope' o~scope~id 'cself' o~cself
say 'message' o~message 'margs' o~margs(1, 2)~items

-- The helpers, outside a call.
say 'helpers' hmhelpers()

-- What only C++ can write.
if @CXX@ then do
  say 'exists' hmexists() hmexists(1) hmexists(, 'b') hmexists(1, 'b', a) -
    hmexistsaftername() hmexistsaftername(1)
  say 'assign' hmassign()
  say 'convert' hmconvert()
  say 'widen' (hmwiden('w') == 'w')
end

-- The classic function, the tables and the package entry.
say 'classic' hmclassic()
say 'rows' hmtablerows()
say 'package' hmpackage()
say 'platform' hmplatform()

::requires '@LIB@' LIBRARY

::class hm
::method init
  expose cself
  cself = hmpointer()
::method msum0 external "LIBRARY @LIB@ HmMSum0"
::method msum1 external "LIBRARY @LIB@ HmMSum1"
::method msum2 external "LIBRARY @LIB@ HmMSum2"
::method msum3 external "LIBRARY @LIB@ HmMSum3"
::method msum4 external "LIBRARY @LIB@ HmMSum4"
::method msum5 external "LIBRARY @LIB@ HmMSum5"
::method msum6 external "LIBRARY @LIB@ HmMSum6"
::method msum7 external "LIBRARY @LIB@ HmMSum7"
::method msum8 external "LIBRARY @LIB@ HmMSum8"
::method msum9 external "LIBRARY @LIB@ HmMSum9"
::method msum10 external "LIBRARY @LIB@ HmMSum10"
::method mrow external "LIBRARY @LIB@ HmMRow"
::method self external "LIBRARY @LIB@ HmSelf"
::method super external "LIBRARY @LIB@ HmSuper"
::method scope external "LIBRARY @LIB@ HmScope"
::method cself external "LIBRARY @LIB@ HmCself"
::method message external "LIBRARY @LIB@ HmMessage"
::method margs external "LIBRARY @LIB@ HmMArgs"
