say BoolHandles('argument_true', .true, .nil)
say BoolHandles('argument_false', .false, .nil)
say BoolHandles('hasentry_missing', .nil, .nil)
say BoolHandles('hasentry_present', .nil, .nil)
say BoolHandles('compare', .nil, .nil)
say BoolHandles('isa', .object~new, .object)
say BoolHandles('rexx_method', .yes~new, .nil)
::requires 'booleans' LIBRARY
::class yes
::method yes
  return .true
