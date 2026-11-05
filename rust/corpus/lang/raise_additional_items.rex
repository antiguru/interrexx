signal on syntax name caught
call raiser
say 'after additional'
call arrayraiser
say 'after array'
exit
caught:
say 'caught' condition('o')~code
exit
raiser:
raise user zz additional (.array~of(.bad~new)) return
arrayraiser:
raise user zz array (.bad~new) return
::class bad
::method defaultname
raise syntax 98.900 array('unwanted rendering')
