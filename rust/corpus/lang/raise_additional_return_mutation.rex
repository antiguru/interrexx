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
a = .array~of('old')
raise syntax 98.900 additional (a) return replace(a)
exit
caught:
say condition('o')~message
exit
replace:
use arg a
a[1] = 'new'
return 1
