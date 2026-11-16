# Sourced by the scripts that compile against the public API headers (build-testbinaries.sh,
# build-extensions.sh, build-spec-fixture.sh): the C++ compiler they use, the flags every such
# compile shares, and the stamp of the headers. Expects $api, the directory of the headers
# (this repository's include/); sets $api_cxx and the array api_flags, and defines api_stamp.
api_cxx=c++
api_flags=(-I"$api" -I"$api/platform/unix" -fPIC)

# One line naming the content of every header under $api, for a build stamp: a header edit,
# addition or removal changes it, with or without a git checkout around the headers.
api_stamp() {
    (cd "$api" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 "${api_sha256[@]}" | "${api_sha256[@]}")
}

# The host's shared-library suffix, which interrexx looks a library up by (`lib<name>.so`, or
# `lib<name>.dylib` on macOS), and the versioned names the two API libraries run under.
case $(uname -s) in
    Darwin)
        api_dylib=dylib
        api_versioned() { echo "$1.4.dylib"; }
        ;;
    *)
        api_dylib=so
        api_versioned() { echo "$1.so.4"; }
        ;;
esac

# Takes an exclusive lock on the open descriptor $1 until the script exits: flock(1) where the
# host has it, else the same flock(2) through Perl, which macOS ships. The lock belongs to the
# open file, which this shell holds, so it outlives the Perl process.
api_lock() {
    if command -v flock >/dev/null; then
        flock "$1"
    else
        perl -e 'use Fcntl ":flock"; open(my $f, ">&=", $ARGV[0]) or die $!; flock($f, LOCK_EX) or die $!' "$1"
    fi
}

# One line per file naming its path, size and modification time, for a build stamp: the command,
# for xargs, and a function over its arguments. GNU stat where the host has it, else BSD's.
if stat -c %n / >/dev/null 2>&1; then
    api_stat=(stat -c '%n %s %.9Y')
else
    api_stat=(stat -f '%N %z %Fm')
fi
api_file_stamp() { "${api_stat[@]}" "$@"; }

# The SHA-256 command, which prints as sha256sum does: sha256sum where the host has it, else the
# shasum macOS ships.
if command -v sha256sum >/dev/null; then api_sha256=(sha256sum); else api_sha256=(shasum -a 256); fi

# The libraries the loader resolves for the product $1, one per line: ldd on ELF hosts, the
# load commands otool lists on macOS.
api_linked_libraries() {
    if [ "$api_dylib" = dylib ]; then otool -L "$1" | tail -n +2; else ldd "$1"; fi
}
