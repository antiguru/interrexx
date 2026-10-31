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
src = .array~of('say "prologue"', '::class c', '::constant k (self)', '::method defaultname class', 'raise syntax 98.900 array("boom")', '::options trace r')
p = .package~new('bad', src)
say 'unexpected'
exit
caught:
trace off
say 'caught' condition('o')~code
say 'after'
exit
