/* The command family's probe. Every `LIBRARY REXX` entry point this family
   owns is built, so there is no refusal here to pin -- the test that runs these
   programs skips a family whose registry rows are all implemented. The program
   stays because every family owns one. */
say 'main'
say .k~probe

::class k

::method probe class external 'LIBRARY REXX cmdhandler_environments'
