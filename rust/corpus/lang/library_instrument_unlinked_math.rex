/* rxmathc names the harness's link to the oracle's C rxmath, which only
   {instruments} on the library path finds. Without it neither side can
   load the library: this crate does not answer rxmathc with its own Rust
   rxmath, so the programs naming rxmathc take the shared-library path. */
say 'not reached' sq(4)
::routine sq external "LIBRARY rxmathc RxCalcSqrt"
