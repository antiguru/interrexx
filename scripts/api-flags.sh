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

# What the host's linker and loader do differently, one definition per host:
#
# * api_dylib: the shared-library suffix, which interrexx looks a library up by (`lib<name>.so`,
#   or `lib<name>.dylib` on macOS), and api_versioned, the versioned names the two API libraries
#   run under.
# * api_rpath DIR: sets the array api_rpath_flags to the link flags that make a product search DIR
#   first for the libraries it needs: DT_RPATH on ELF hosts, which the loader consults before
#   LD_LIBRARY_PATH, and LC_RPATH on macOS, which the API libraries' `@rpath` install names name.
# * api_keep_needed: the link flags that record every library named with -l as needed, although
#   the product may call into only one. GNU ld drops an unused one under --as-needed; Apple's
#   linker records every one unless told to strip them.
# * api_resolves PRODUCT LIBRARY DIR [NAME=VALUE...]: whether the loader resolves the API library
#   LIBRARY (its versioned name) for PRODUCT to DIR/LIBRARY, under the environment given: status
#   0 where it does, 1 where it resolves it elsewhere or not at all, after naming what it found
#   on stderr, and 2 where PRODUCT does not need LIBRARY. ELF hosts ask ldd. On macOS the answer
#   is read from the product's load commands, since dyld resolves only at run time: LIBRARY must
#   be recorded as `@rpath/LIBRARY` and DIR must be the product's one LC_RPATH. The environment
#   given there is not consulted; the names ELF hosts take, LD_LIBRARY_PATH among them, mean
#   nothing to dyld.
case $(uname -s) in
    Darwin)
        api_dylib=dylib
        api_versioned() { echo "$1.4.dylib"; }
        api_rpath() { api_rpath_flags=(-Wl,-rpath,"$1"); }
        api_keep_needed=()
        api_resolves() {
            local product=$1 library=$2 dir=$3 loads rpaths
            loads=$(otool -L "$product" | tail -n +2)
            if ! grep -q "$library" <<<"$loads"; then
                return 2
            fi
            rpaths=$(otool -l "$product" | awk '$2 == "LC_RPATH" { getline; getline; print $2 }')
            if grep -q "^[[:space:]]*@rpath/$library " <<<"$loads" && [ "$rpaths" = "$dir" ]; then
                return 0
            fi
            { grep "$library" <<<"$loads"; echo "  LC_RPATH: ${rpaths:-(none)}"; } >&2
            return 1
        }
        ;;
    *)
        api_dylib=so
        api_versioned() { echo "$1.so.4"; }
        api_rpath() { api_rpath_flags=(-Wl,--disable-new-dtags -Wl,-rpath,"$1"); }
        api_keep_needed=(-Wl,--no-as-needed)
        api_resolves() {
            local product=$1 library=$2 dir=$3 resolved
            shift 3
            resolved=$(env "$@" ldd "$product")
            if ! grep -q "^[[:space:]]*$library " <<<"$resolved"; then
                return 2
            fi
            if grep -q "^[[:space:]]*$library => $dir/$library " <<<"$resolved"; then
                return 0
            fi
            grep "$library" <<<"$resolved" >&2
            return 1
        }
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
