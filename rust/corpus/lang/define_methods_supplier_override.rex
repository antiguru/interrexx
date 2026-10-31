/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

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
