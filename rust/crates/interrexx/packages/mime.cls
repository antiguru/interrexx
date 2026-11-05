/*----------------------------------------------------------------------------*/
/* mime.cls: MIME message parts.                                              */
/*                                                                            */
/* Written for this crate from the extension's documented behaviour;          */
/* its line numbers are its own (Deviation 89).                               */
/*----------------------------------------------------------------------------*/

/* The header lines both classes write after their Content-Type, each only    */
/* where its value is not the null string.                                    */
::routine headers
  use arg part
  crlf = '0d0a'x
  text = ''
  if part~encoding \== '' then
    text = text || 'Content-Transfer-Encoding:' part~encoding || crlf
  if part~id \== '' then
    text = text || 'Content-ID:' part~id || crlf
  if part~description \== '' then
    text = text || 'Content-Description:' part~description || crlf
  if part~disposition \== '' then
    text = text || 'Content-Disposition:' part~disposition || crlf
  return text


::class 'mimepart' public

::constant crlf '0d0a'x

::attribute type
::attribute encoding
::attribute id
::attribute description
::attribute disposition
::attribute content get

::method init
  expose type encoding id description disposition content
  use strict arg type = 'text/plain'
  encoding = ''
  id = ''
  description = ''
  disposition = ''
  content = ''

/* A line that is exactly a period is stored doubled, as SMTP's DATA needs.  */
::method addContent
  expose content
  use strict arg line
  if line == '.' then line = '..'
  content = content || line || self~crlf

::method string
  expose type content
  text = 'MIME-Version: 1.0' || self~crlf
  if type \== '' then
    text = text || 'Content-Type:' type || self~crlf
  return text || headers(self) || self~crlf || content || self~crlf


::class 'mimemultipart' public

::constant crlf '0d0a'x

::attribute type
::attribute encoding
::attribute id
::attribute description
::attribute disposition

::method init
  expose type encoding id description disposition parts
  use strict arg
  type = 'multipart/mixed'
  encoding = ''
  id = ''
  description = ''
  disposition = ''
  parts = .array~new

::method addPart
  expose parts
  use strict arg part
  if \part~isA(.mimepart) then
    raise syntax 93.948 array('MIMEPART')
  parts~append(part)

::method string
  expose type parts
  boundary = '===ooRexx=Mime=Part===' || c2x(date('B') || time('L')) || '=='
  text = 'MIME-Version: 1.0' || self~crlf
  if type \== '' then
    text = text || 'Content-Type:' type || '; boundary="' || boundary || '"' || self~crlf
  text = text || headers(self) || self~crlf
  if parts~items = 0 then return text
  do part over parts
    text = text || '--' || boundary || self~crlf || part~string || self~crlf
  end
  return text || '--' || boundary || '--' || self~crlf
