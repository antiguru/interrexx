/* Message~notify takes one MessageNotification; messageComplete and
   triggered send the message and answer nothing. */
m = .message~new('abc', 'reverse')
signal on syntax name s1
m~notify(1)
s1:
  say condition('o')~code
signal on syntax name s2
m~notify
s2:
  say condition('o')~code
m~messageComplete
say 'sent' m~result
signal on syntax name s3
say m~triggered(5)
s3:
  say condition('o')~code
