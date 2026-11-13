# Sourced by the scripts that compile against ooRexx's public API headers (build-testbinaries.sh,
# build-spec-fixture.sh): the C++ compiler they use and the flags every such compile shares.
# Expects $api, the directory of the headers; sets $api_cxx and the array api_flags.
api_cxx=c++
api_flags=(-I"$api" -I"$api/platform/unix" -fPIC)
