/* Untraced ADDITIONAL renders nothing for a trace line. DESCRIPTION takes
   its required string at RAISE under Deviation 83. A SYNTAX code uses the
   RC value's string value. */

target = .object~subclass('Target')
.local~target = target
methods = .Coll~new
methods['outer'] = 'return 1'
target~defineMethods(methods)
say 'installed' target~new~outer target~new~inner target~new~nested
plain = .directory~new
plain['plain'] = 'return 4'
target~defineMethods(plain)
target~defineMethods(.Protocol~new)
say 'adjacent' target~new~plain target~new~protocol
exit
::class Coll subclass Directory
::method supplier
say 'supplier'
.local~target~define('inner', 'return 2')
.local~target~defineMethods(self~nestedMethods)
return self~supplier:super
::method nestedMethods
result = .directory~new
result['nested'] = 'return 3'
return result
::class Protocol
::method supplier
return .supplier~new(.array~of('return 5'), .array~of('protocol'))
