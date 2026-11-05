supplier = .directory~method('SUPPLIER')~copy
supplier~setProtected
.ProtectedCollection~define('SUPPLIER', supplier)
manager = .Manager~new
.context~package~setSecurityManager(manager)
call install .ProtectedCollection~new
say 'installed' .Target~new~m
exit
::routine install
use arg methods
.Target~defineMethods(methods)
::class Target
::class ProtectedCollection subclass Directory
::class Manager
::method unknown
use arg name, args
if name == 'METHOD', args[1]~name == 'SUPPLIER' then do
  say 'supplier checkpoint'
  args[1]~result = .supplier~new(.array~of('return 9'), .array~of('m'))
  return 1
end
return 0
