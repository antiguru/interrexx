# Sourced by the scripts that compile against the public API headers (build-testbinaries.sh,
# build-extensions.sh, build-spec-fixture.sh): the C++ compiler they use, the flags every such
# compile shares, and the stamp of the headers. Expects $api, the directory of the headers
# (this repository's include/); sets $api_cxx and the array api_flags, and defines api_stamp.
api_cxx=c++
api_flags=(-I"$api" -I"$api/platform/unix" -fPIC)

# One line naming the content of every header under $api, for a build stamp: a header edit,
# addition or removal changes it, with or without a git checkout around the headers.
api_stamp() {
    (cd "$api" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum | sha256sum)
}
