/* Run by the driver hostwait through RexxStart: starts a thread of the extension that attaches
   once the program has ended, and returns at once.
   spec: native.threads.end */
call AtAttachLater 300
return 'returned'
::requires 'attach' LIBRARY
