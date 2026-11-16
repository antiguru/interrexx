/* A classic function is registered with RxFuncAdd, called and dropped with RxFuncDrop. A wait
   that nothing ends afterwards is not refused: a thread the library started could still end it.
   spec: conc.activities.deviation.20 */
say RxFuncAdd('AtClassicOne', 'attachclassic', 'AtClassicOne')
say AtClassicOne()
say RxFuncDrop('AtClassicOne')
sem = .eventsemaphore~new
say 'waiting'
sem~wait
say 'never'
