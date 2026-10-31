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

do condition over .array~of('user zz', 'syntax 40.4')
  do operand over .array~of('additional (.array~of(bad))', 'array (bad)')
    do mode over .array~of('off', 'r', 'i')
      say condition operand mode runCase(condition, operand, mode)
    end
  end
end
exit
::routine runCase
use arg condition, operand, mode
source = .array~of('signal on syntax name caught', 'bad = .Bad~new', 'trace 'mode, ,
  'raise 'condition' 'operand' return "raised"', 'caught:', 'trace off', ,
  "return condition('o')~code", '::class Bad', '::method defaultname', ,
  'raise syntax 98.900 array("rendered")')
return .routine~new('raise-case', source)~call
