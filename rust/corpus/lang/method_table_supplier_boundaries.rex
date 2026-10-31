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

signal on syntax name caught
.Target~defineMethods(.BadCollection~new)
say 'unexpected'
exit
caught:
say 'atomic' .Target~new~hasMethod('fresh') .Target~new~old
child = .object~subclass('Child', .class, .GoodCollection~new)
say 'class method' child~m
object = .object~enhanced(.GoodCollection~new)
say 'enhanced' object~m
plain = .stringTable~new
plain['plain'] = 'return 4'
.Target~defineMethods(plain)
say 'plain' .Target~new~plain
exit
::class Target
::method old
return 8
::class BadCollection subclass Directory
::method supplier
say 'bad supplier'
return .supplier~new(.array~of('return 1', 'if'), .array~of('fresh', 'bad'))
::class GoodCollection subclass Directory
::method supplier
say 'good supplier'
return .supplier~new(.array~of('return 7'), .array~of('m'))
