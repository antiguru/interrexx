/* The extension's C API registration must reach this running interpreter. */
if installCliFunction() <> 0 then exit 2
if cliRegistered() \== 'shared registration' then exit 3
exit 0

::requires 'cli-registration' library
