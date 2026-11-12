/* LocalServer: the library's builder of the standard `.local` entries. It is
   not public and no spec page names it; the interpreter builds the same
   entries itself before a program runs. A new instance builds the three
   standard streams and keeps them, then sends itself initInstance, which puts
   the streams it keeps into `.local` and builds the five monitors over them
   and the session queue. Each entry replaces the one of its name. */

::class "LocalServer"

::method init
  expose stdin stdout stderr
  stdin = self~interrexx_standardStream('STDIN')
  stdout = self~interrexx_standardStream('STDOUT')
  stderr = self~interrexx_standardStream('STDERR')
  self~initInstance

::method initInstance
  expose stdin stdout stderr
  .local['STDIN'] = stdin
  .local['STDOUT'] = stdout
  .local['STDERR'] = stderr
  .local['INPUT'] = monitor(stdin, 'The INPUT monitor')
  .local['DEBUGINPUT'] = monitor(.local['INPUT'], 'The DEBUG INPUT monitor')
  .local['OUTPUT'] = monitor(stdout, 'The OUTPUT monitor')
  .local['ERROR'] = monitor(stderr, 'The ERROR monitor')
  .local['TRACEOUTPUT'] = monitor(.local['ERROR'], 'The TRACE OUTPUT monitor')
  .local['STDQUE'] = .RexxQueue~new('SESSION')
  return

monitor: procedure
  use arg destination, name
  built = .Monitor~new(destination)
  built~objectName = name
  return built

::method interrexx_standardStream private external 'LIBRARY REXX interrexx_standard_stream'
