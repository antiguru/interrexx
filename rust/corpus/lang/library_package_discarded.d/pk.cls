::options digits 12
::routine pkr public
  say 'pkr ran'
::method um
  return 1
::resource res
resource line
::END
::class Re public
::method doparse external "LIBRARY rxregexpc RegExp_Parse"
::method missing external "LIBRARY rxregexpc NoSuchEntry"
