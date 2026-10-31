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
call raiser
say 'after additional'
call arrayraiser
say 'after array'
exit
caught:
say 'caught' condition('o')~code
exit
raiser:
raise user zz additional (.array~of(.bad~new)) return
arrayraiser:
raise user zz array (.bad~new) return
::class bad
::method defaultname
raise syntax 98.900 array('unwanted rendering')
