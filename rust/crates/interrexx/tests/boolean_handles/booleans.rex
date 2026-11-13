say BoolHandles('argument_true', .true, .nil)
say BoolHandles('argument_false', .false, .nil)
say BoolHandles('hasentry_missing', .nil, .nil)
say BoolHandles('hasentry_present', .nil, .nil)
say BoolHandles('compare', .nil, .nil)
say BoolHandles('isa', .object~new, .object)
say BoolHandles('rexx_method', .yes~new, .nil)
say BoolHandles('argument_routine_true', LogicalAnswer(5), .nil)
say BoolHandles('argument_routine_false', LogicalAnswer(0), .nil)
say BoolHandles('argument_method_true', .logicalholder~new~LogicalMethod, .nil)
::requires 'booleans' LIBRARY
::class logicalholder
::method logicalmethod external 'LIBRARY booleans LogicalMethod'
::class yes
::method yes
  return .true
