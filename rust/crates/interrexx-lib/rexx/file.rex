/* The File class. Spec rules: class.file.*.

   An instance keeps two strings: PATH, the path as init made it
   (r[class.file.path]), and QUALIFIEDPATH, its absolute path, worked out
   once when the instance is made (r[class.file.absolute]). Every question
   about the file system goes to a private method bound to a native entry
   point, which takes the absolute path as its first argument and does not
   look at the receiver. The private interrexx_string, on both sides, answers
   its argument as a plain string when it is one and .nil otherwise, sending
   it nothing; a string argument is read through it and, for any other
   object, through the answer of request('STRING') (r[class.index.string]). */

::class "File" public inherit Comparable Orderable

-- r[class.file.separators]: a constant is both an instance and a class
-- method, so each answers the same on both sides.
::constant separator "/"
::constant pathSeparator ":"

-- r[class.file.new], r[class.file.new.sends], r[class.file.string-args]
::method init
  expose path qualifiedPath
  use strict arg given, dir = .nil
  -- A File as dir is read by its absolute path, before the first
  -- separator is sent; any other dir is read as a string argument.
  if .nil \== dir then do
    if dir~isA(.File) then dirText = dir~absolutePath
    else do
      dirText = self~interrexx_string(dir)
      if .nil == dirText then dirText = self~interrexx_string(dir~request('STRING'))
      if .nil == dirText then raise syntax 88.909 array(1)
    end
  end
  separator = self~separator
  text = self~interrexx_string(given)
  if .nil == text then text = self~interrexx_string(given~request('STRING'))
  if .nil == text then raise syntax 88.909 array(1)
  -- Trailing separators go, but a path that is nothing else keeps one.
  size = separator~length
  if size > 0 then
    do while text~length > size & text~right(size) == separator
      text = text~left(text~length - size)
    end
  if .nil \== dir then do
    separator = self~separator
    size = separator~length
    if size > 0 then
      do while dirText~length > size & dirText~right(size) == separator
        dirText = dirText~left(dirText~length - size)
      end
    -- The path loses its leading separators, and one separator joins it to
    -- dir unless dir ends in one.
    separator = self~separator
    size = separator~length
    if size > 0 then do
      do while text~left(size) == separator
        text = text~substr(size + 1)
      end
      if dirText~right(size) \== separator then dirText = dirText || separator
    end
    text = dirText || text
  end
  -- r[class.file.absolute]: the empty path's absolute path is empty.
  if text == '' then absolute = ''
  else absolute = self~qualifyImpl(text)
  path = text
  qualifiedPath = absolute

-- r[class.file.path]
::attribute path get

::method string
  expose path
  use strict arg
  return path

::method makeString
  expose qualifiedPath
  use strict arg
  return qualifiedPath

-- r[class.file.absolute]
::method absolutePath
  expose qualifiedPath
  use strict arg
  return qualifiedPath

::method absoluteFile
  expose qualifiedPath
  use strict arg
  return self~class~new(qualifiedPath)

-- r[class.file.iscasesensitive]
::method isCaseSensitive
  expose qualifiedPath
  use strict arg
  return self~isCaseSensitiveImpl(qualifiedPath)

-- r[class.file.name], r[class.file.name.sends]
::method name
  expose qualifiedPath
  use strict arg
  separator = self~separator
  at = qualifiedPath~lastPos(separator)
  if at == 0 then return qualifiedPath
  return qualifiedPath~substr(at + separator~length)

-- An extension is what follows the last period of the name, unless that
-- period is its first or its last character.
::method extension
  use strict arg
  separator = self~separator
  name = self~name
  at = name~lastPos('.')
  if at <= 1 | at == name~length then return ''
  return name~substr(at + 1)

-- r[class.file.parent], r[class.file.parent.sends]
::method parent
  expose qualifiedPath
  use strict arg
  separator = self~separator
  if qualifiedPath == '' | qualifiedPath == separator then return .nil
  at = qualifiedPath~lastPos(separator)
  if at == 0 then return .nil
  if at == 1 then return separator
  return qualifiedPath~left(at - 1)

::method parentFile
  use strict arg
  parent = self~parent
  if .nil == parent then return .nil
  return self~class~new(parent)

-- r[class.file.exists]
::method exists
  expose qualifiedPath
  use strict arg
  return self~existsImpl(qualifiedPath)

::method isFile
  expose qualifiedPath
  use strict arg
  return self~isFileImpl(qualifiedPath)

::method isDirectory
  expose qualifiedPath
  use strict arg
  return self~isDirectoryImpl(qualifiedPath)

::method isHidden
  expose qualifiedPath
  use strict arg
  return self~isHiddenImpl(qualifiedPath)

::method length
  expose qualifiedPath
  use strict arg
  return self~lengthImpl(qualifiedPath)

::method canRead
  expose qualifiedPath
  use strict arg
  return self~canReadImpl(qualifiedPath)

::method canWrite
  expose qualifiedPath
  use strict arg
  return self~canWriteImpl(qualifiedPath)

-- r[class.file.delete], r[class.file.delete.isdirectory]
::method delete
  expose qualifiedPath
  use strict arg
  if self~isDirectory then return self~deleteDir(qualifiedPath)
  return self~deleteFile(qualifiedPath)

-- r[class.file.makedir]
::method makeDir
  expose qualifiedPath
  use strict arg
  return self~makeDirImpl(qualifiedPath)

::method makeDirs
  use strict arg
  if self~exists then return 0
  if self~makeDir then return 1
  parent = self~parentFile
  if .nil == parent then return 0
  parent~makeDirs
  return self~makeDir

-- r[class.file.renameto], r[class.file.renameto.directory]
::method renameTo
  expose qualifiedPath
  use strict arg destination
  if \destination~isA(.File) then raise syntax 88.914 array('destination', 'File')
  return self~renameToImpl(qualifiedPath, destination~absolutePath)

-- r[class.file.permissions]: neither answers anything.
::method setReadOnly
  expose qualifiedPath
  use strict arg
  self~setReadOnlyImpl(qualifiedPath)

::method setWritable
  expose qualifiedPath
  use strict arg
  self~setWritableImpl(qualifiedPath)

-- r[class.file.lastmodified]: the entry point answers local civil
-- microseconds, or a sentinel for a file it cannot stat. It is compared
-- as text, since the default digits cannot hold it.
::attribute lastModified get
  expose qualifiedPath
  use strict arg
  stamp = self~getLastModifiedImpl(qualifiedPath)
  if stamp == '-999999999999999999' then return .nil
  return .DateTime~new(stamp)

-- r[class.file.lastmodified-set]: the fields of the argument, read in the
-- host's zone, which is what fullDate answers.
::attribute lastModified set
  expose qualifiedPath
  use strict arg date
  if \date~isA(.DateTime) then raise syntax 88.914 array('date', 'DateTime')
  self~setLastModifiedImpl(qualifiedPath, date~fullDate)

-- r[class.file.lastaccessed], r[class.file.lastaccessed-set]
::attribute lastAccessed get
  expose qualifiedPath
  use strict arg
  stamp = self~getLastAccessedImpl(qualifiedPath)
  if stamp == '-999999999999999999' then return .nil
  return .DateTime~new(stamp)

::attribute lastAccessed set
  expose qualifiedPath
  use strict arg date
  if \date~isA(.DateTime) then raise syntax 88.914 array('date', 'DateTime')
  self~setLastAccessedImpl(qualifiedPath, date~fullDate)

-- r[class.file.list]
::method list
  expose qualifiedPath
  use strict arg
  return self~listImpl(qualifiedPath)

::method listFiles
  use strict arg
  names = self~list
  if .nil == names then return .nil
  class = self~class
  files = .array~new(names~items)
  do name over names
    files~append(class~new(name, self))
  end
  return files

-- r[class.file.compareto], r[class.file.deviation.126]: an argument that is
-- no File raises before anything else is sent.
::method compareTo
  use strict arg other
  if \other~isA(.File) then raise syntax 88.914 array('other', 'File')
  caseSensitive = self~isCaseSensitive
  mine = self~absolutePath
  theirs = other~absolutePath
  if \caseSensitive then do
    mine = mine~upper
    theirs = theirs~upper
  end
  if mine == theirs then return 0
  if mine << theirs then return -1
  return 1

-- r[class.file.hashcode]
::method hashCode
  expose qualifiedPath
  use strict arg
  return qualifiedPath~hashCode

-- Private helpers bound to native entry points.
::method interrexx_string private external "LIBRARY REXX interrexx_file_string"
::method qualifyImpl private external "LIBRARY REXX file_qualify"
::method existsImpl private external "LIBRARY REXX file_exists"
::method isFileImpl private external "LIBRARY REXX file_isFile"
::method isDirectoryImpl private external "LIBRARY REXX file_isDirectory"
::method isHiddenImpl private external "LIBRARY REXX file_isHidden"
::method lengthImpl private external "LIBRARY REXX file_length"
::method canReadImpl private external "LIBRARY REXX file_can_read"
::method canWriteImpl private external "LIBRARY REXX file_can_write"
::method isCaseSensitiveImpl private external "LIBRARY REXX this_file_case_sensitive"
::method deleteDir private external "LIBRARY REXX file_delete_directory"
::method deleteFile private external "LIBRARY REXX file_delete_file"
::method makeDirImpl private external "LIBRARY REXX file_make_dir"
::method renameToImpl private external "LIBRARY REXX file_rename"
::method setReadOnlyImpl private external "LIBRARY REXX file_set_read_only"
::method setWritableImpl private external "LIBRARY REXX file_set_writable"
::method getLastModifiedImpl private external "LIBRARY REXX file_get_last_modified"
::method setLastModifiedImpl private external "LIBRARY REXX file_set_last_modified"
::method getLastAccessedImpl private external "LIBRARY REXX file_get_last_accessed"
::method setLastAccessedImpl private external "LIBRARY REXX file_set_last_accessed"
::method listImpl private external "LIBRARY REXX file_list"

-- r[class.file.iscasesensitive.class-side], r[class.file.listroots]
::method isCaseSensitive class external "LIBRARY REXX file_case_sensitive"
::method listRoots class external "LIBRARY REXX file_list_roots"

-- r[class.file.temporarypath]
::method temporaryPath class
  use strict arg
  return self~new(self~file_temporary_path)

-- r[class.file.searchpath], r[class.file.searchpath.relative],
-- r[class.file.searchpath.order]: the path is read before the name, and
-- every directory of an array path before any is searched.
::method searchPath class
  use strict arg name, path = .nil
  if arg(2, 'O') then do
    separator = self~pathSeparator
    list = value('PATH', , 'ENVIRONMENT')
    directories = .nil
  end
  else do
    list = self~interrexx_string(path)
    directories = .nil
    if .nil \== list then do
      if list \== '' then separator = self~pathSeparator
    end
    else do
      items = path~request('ARRAY')
      if .nil == items then raise syntax 98.913 array(path)
      if \items~isA(.Array) then raise syntax 98.913 array(path)
      directories = .array~new
      do item over items
        if item~isA(.File) then text = item~absolutePath
        else do
          text = self~interrexx_string(item)
          if .nil == text then text = self~interrexx_string(item~request('STRING'))
          if .nil == text then raise syntax 88.909 array(1)
        end
        directories~append(text)
      end
    end
  end
  text = self~interrexx_string(name)
  if .nil == text then text = self~interrexx_string(name~request('STRING'))
  if .nil == text then raise syntax 88.909 array(1)
  if text == '' | text~verify('*?', 'M') > 0 then return .nil
  if .nil == directories then found = self~searchPathImpl(text, list)
  else do
    -- The entry point answers an absolute name whatever the list is.
    if directories~items == 0 then found = self~searchPathImpl(text, '')
    else do directory over directories
      found = self~searchPathImpl(text, directory)
      if .nil \== found then leave
    end
  end
  if .nil == found then return .nil
  return .File~new(found)

-- r[class.file.readchars]
::method readChars class
  use strict arg name
  if name~isA(.File) then text = name~absolutePath
  else do
    text = self~interrexx_string(name)
    if .nil == text then text = self~interrexx_string(name~request('STRING'))
    if .nil == text then raise syntax 93.938 array(1)
  end
  content = ''
  stream = .Stream~new(text)
  if stream~open('READ')~left(6) == 'READY:' then do
    size = stream~chars
    if size > 0 then content = stream~charIn(1, size)
    stream~close
  end
  return content

-- r[class.file.readlines]: a line ends at a line feed, and one carriage
-- return just before it goes with it.
::method readLines class
  use strict arg name
  if name~isA(.File) then text = name~absolutePath
  else do
    text = self~interrexx_string(name)
    if .nil == text then text = self~interrexx_string(name~request('STRING'))
    if .nil == text then raise syntax 93.938 array(1)
  end
  -- The content is read here rather than by sending readChars, which would
  -- be a message to the receiver (r[class.file.sends]).
  content = ''
  stream = .Stream~new(text)
  if stream~open('READ')~left(6) == 'READY:' then do
    size = stream~chars
    if size > 0 then content = stream~charIn(1, size)
    stream~close
  end
  lines = .array~new
  linefeed = '0a'x
  return_ = '0d'x
  do while content \== ''
    at = content~pos(linefeed)
    if at == 0 then do
      lines~append(content)
      leave
    end
    line = content~left(at - 1)
    if line~right(1) == return_ then line = line~left(line~length - 1)
    lines~append(line)
    content = content~substr(at + 1)
  end
  return lines

-- r[class.file.writechars]: the mode is checked before the name is read,
-- and the string is read last, by its string value.
::method writeChars class
  use strict arg name, string, mode = 'A'
  if arg(3, 'E') then do
    letter = mode~strip~left(1)~upper
    if letter \== 'A' & letter \== 'R' then
      raise syntax 88.916 array('mode', "'A[ppend]' or 'R[eplace]'", mode)
  end
  else letter = 'A'
  if name~isA(.File) then text = name~absolutePath
  else do
    text = self~interrexx_string(name)
    if .nil == text then text = self~interrexx_string(name~request('STRING'))
    if .nil == text then raise syntax 93.938 array(1)
  end
  value = self~interrexx_string(string)
  if .nil == value then value = self~interrexx_string(string~request('STRING'))
  if .nil == value then value = string~string
  stream = .Stream~new(text)
  if letter == 'R' then opened = stream~open('WRITE REPLACE')
  else opened = stream~open('WRITE APPEND')
  if opened~left(6) \== 'READY:' then return
  if value \== '' then stream~charOut(value)
  stream~close

-- r[class.file.writelines], r[class.file.deviation.118]: the collection, the
-- mode and the name are checked in that order.
::method writeLines class
  use strict arg name, collection, mode = 'A'
  if \collection~isA(.OrderedCollection) then
    raise syntax 88.914 array('collection', 'OrderedCollection')
  if arg(3, 'E') then do
    letter = mode~strip~left(1)~upper
    if letter \== 'A' & letter \== 'R' then
      raise syntax 88.916 array('mode', "'A[ppend]' or 'R[eplace]'", mode)
  end
  else letter = 'A'
  if name~isA(.File) then text = name~absolutePath
  else do
    text = self~interrexx_string(name)
    if .nil == text then text = self~interrexx_string(name~request('STRING'))
    if .nil == text then raise syntax 93.938 array(1)
  end
  items = collection~request('ARRAY')
  if .nil == items then raise syntax 98.913 array(collection)
  if \items~isA(.Array) then raise syntax 98.913 array(collection)
  content = .MutableBuffer~new
  do i = 1 to items~size
    if \items~hasIndex(i) then iterate
    item = items[i]
    value = self~interrexx_string(item)
    if .nil == value then value = self~interrexx_string(item~request('STRING'))
    if .nil == value then value = item~string
    content~append(value)
    content~append('0a'x)
  end
  stream = .Stream~new(text)
  if letter == 'R' then opened = stream~open('WRITE REPLACE')
  else opened = stream~open('WRITE APPEND')
  if opened~left(6) \== 'READY:' then return
  if content~length > 0 then stream~charOut(content~string)
  stream~close

::method interrexx_string class private external "LIBRARY REXX interrexx_file_string"
::method searchPathImpl class private external "LIBRARY REXX file_search_path_impl"
::method file_temporary_path class private external "LIBRARY REXX file_temporary_path"
-- Unsent: the separators are constants. They are kept because user code
-- binds the same entry points by name, so the registry holds them anyway.
::method getSeparator class private external "LIBRARY REXX file_separator"
::method getPathSeparator class private external "LIBRARY REXX file_path_separator"
