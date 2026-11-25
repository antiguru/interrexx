/* Distinguish the override of AT from the ENTRY override that a computed entry sends.
   spec: native.threads.members */
directory = .ProbeDirectory~new
directory['BASE'] = 'seed'
directory~setMethod('COMPUTED', 'return self~entry("BASE")')
say 'native' AtDirectoryAt(directory, 'COMPUTED')
say 'message' directory~at('COMPUTED')

::requires 'attach' library

::class ProbeDirectory subclass Directory
::method at
  say 'AT runs'
  forward class (super)

::method entry
  say 'ENTRY runs' arg(1)
  forward class (super)
