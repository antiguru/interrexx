/* synthetic host B */
::class Gamma public
::method run
  return 'host run'
::class Hidden private
::method secret
  return 'host secret'
::routine uses
  use arg o
  return o~run
