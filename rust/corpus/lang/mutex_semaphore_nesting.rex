/* MutexSemaphore in one activity: re-entrant for its owner, one release per
   acquire. */
m = .MutexSemaphore~new
say 'free release' m~release
say 'acquire' m~acquire m~acquire(0) m~acquire(2) m~acquire(-1) m~acquire(.TimeSpan~fromSeconds(1))
say 'release' m~release m~release m~release m~release m~release m~release
signal on syntax name notNumber
say m~acquire('x')
notNumber:
say 'error' condition('o')~code
say 'held' m~acquire
