/* StreamSupplier: a Supplier over the lines of a stream.
   See spec/src/classes/streamsupplier.md, rules class.streamsupplier.creation,
   class.streamsupplier.entry, class.streamsupplier.next,
   class.streamsupplier.sends, class.streamsupplier.rest and
   class.streamsupplier.deviation.138. */

::class "StreamSupplier" subclass Supplier public

/* r[class.streamsupplier.creation] r[class.streamsupplier.sends]
   The stream is asked its kind, then `state` twice; the private messages are
   not part of the specification. A transient stream is never given a line
   number. For any other stream, the supplier starts at the line its read
   position is in when the second answer is READY, and at line 1 otherwise. It
   is made inside that line when the position is not at its start
   (r[class.stream.deviation.140]); then its first read asks for the rest of
   the line, with no argument. The first `next` is sent to the receiver, so an
   override takes part. */
::method init
  expose stream index valid first plain transient closing
  use arg stream
  transient = (stream~'!query_streamtype' == 'TRANSIENT')
  /* A stream that was not open when the supplier was made is closed again
     when a read finds no line left. */
  closing = (stream~state == 'UNKNOWN')
  state = stream~state
  index = 1
  plain = transient
  if \transient & state == 'READY' then do
    index = stream~'!query_position'('READ LINE') + 0
    plain = self~interrexx_mid_line(stream)
  end
  /* r[class.streamsupplier.deviation.138]: what allItems and allIndexes
     collected for an earlier stream is dropped. */
  self~interrexx_streamcollected(.nil)
  valid = 1
  first = 1
  self~next
  return

/* r[class.streamsupplier.entry] */
::method available
  expose valid
  if arg() > 0 then raise syntax 93.902 array (0)
  return valid

/* r[class.streamsupplier.entry] */
::method index
  expose index valid
  if arg() > 0 then raise syntax 93.902 array (0)
  if \valid then raise syntax 93.937
  return index

/* r[class.streamsupplier.entry] */
::method item
  expose item valid
  if arg() > 0 then raise syntax 93.902 array (0)
  if \valid then raise syntax 93.937
  return item

/* r[class.streamsupplier.next]: reads the line after the current one, by its
   number. The read init asks for is the same code, at the line init chose.
   A NOTREADY from the stream ends the supplier and is not passed on. */
::method next
  expose stream index item valid first plain transient closing
  if arg() > 0 then raise syntax 93.902 array (0)
  if first then first = 0
  else do
    if \valid then raise syntax 93.937
    index += 1
  end
  signal on notready name ended
  if plain then item = stream~lineIn
  else item = stream~lineIn(index)
  plain = transient
  return
ended:
  valid = 0
  item = .nil
  if closing then stream~close
  return

/* Whether the read position of a stream is inside a line, not at its start. */
::method interrexx_mid_line private external 'LIBRARY REXX interrexx_stream_mid_line'

/* The cache allItems and allIndexes keep, as SupplierMixin's own private
   method reads it; with .nil it drops what was collected. */
::method interrexx_streamcollected private external 'LIBRARY REXX interrexx_supplier_collected'
