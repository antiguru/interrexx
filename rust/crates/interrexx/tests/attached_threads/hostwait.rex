/* Run by the driver hostwait through RexxStart: a wait on an event semaphore that nothing in the
   program posts, ended by the halt the host sends.
   spec: conc.activities.deviation.20 */
signal on halt
.eventsemaphore~new~wait
return 'not halted'
halt:
return 'halted'
