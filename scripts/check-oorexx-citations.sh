#!/bin/sh
# Fails when a tracked file under rust/ cites a source file of ooRexx: a C++
# or Rexx class-library file name such as `Setup.cpp`, `ClassClass.hpp` or
# `CoreClasses.orx`. interrexx's text states behaviour; it does not point into
# the implementation it was checked against. State what the oracle does, or
# the spec rule that requires it, instead.
#
# Not flagged:
#   * names of files this repository tracks itself (its own C++ probe sources);
#   * the files in ALLOW below. They read ooRexx's sources while building or
#     testing, hold oracle output verbatim, or carry a literal string the code
#     compares. Issues #388 and #47 replace the build-time readers; shrink the
#     list as that happens and do not add to it.
#
# The patterns are plain regular expressions over file contents. Submodules are
# not searched.
set -eu
cd "$(git rev-parse --show-toplevel)"

# One path (or path prefix ending in /) per line.
ALLOW='
rust/corpus/docs/directive-options.txt
rust/corpus/collection-scopes.tsv
rust/corpus/introspection-scopes.tsv
rust/corpus/oracle-crashes.txt
rust/corpus/refusal-sites.tsv
rust/corpus/lang/method_introspection.rex
rust/corpus/lang/package_rexx.rex
rust/crates/interrexx-parse/tests/sourceline_oracle/method_introspection.txt
rust/crates/interrexx-parse/tests/sourceline_oracle/package_rexx.txt
rust/crates/interrexx/tests/sim_gate/timer_post_wakes_every_waiter.rex
rust/crates/interrexx-classes/build.rs
rust/crates/interrexx-classes/src/native_classes.rs
rust/crates/interrexx-classes/tests/
rust/crates/interrexx-inventory/build.rs
rust/crates/interrexx-lib/build.rs
rust/crates/interrexx-lib/src/lib.rs
rust/crates/interrexx-extract/src/docs.rs
rust/crates/interrexx-extract/src/docs/directives.rs
rust/crates/interrexx-parse/benches/parse.rs
rust/crates/interrexx-parse/src/directive/tests.rs
rust/crates/interrexx-parse/tests/errors.rs
rust/crates/interrexx-api/tests/
rust/crates/interrexx/tests/
rust/crates/interrexx/src/builtin/tests.rs
rust/crates/interrexx/src/dispatch/native/tests.rs
rust/crates/interrexx/src/parse_template/tests.rs
'

own=$(git ls-files -- rust | sed -n 's/.*\.\(cpp\|hpp\|orx\)$/&/p' | sed 's|.*/||' | sort -u)
export OWN="$own" ALLOW
found=$(git ls-files -z -- rust | perl -0 -ne '
  chomp;
  BEGIN {
    %own = map { $_ => 1 } split /\n/, $ENV{OWN};
    @allow = grep { length } split /\n/, $ENV{ALLOW};
  }
  next unless -f $_ && -T $_;
  my $path = $_;
  for my $a (@allow) {
    next LINE if $a eq $path || ($a =~ m{/$} && index($path, $a) == 0);
  }
  open my $f, "<", $path or die "$path: $!";
  local $/ = "\n";
  while (my $line = <$f>) {
    while ($line =~ /([A-Za-z_][A-Za-z0-9_]*\.(?:cpp|hpp|orx))\b/g) {
      next if $own{$1};
      chomp(my $text = $line);
      print "$path:$.: $text\n";
      last;
    }
  }
  close $f;
  LINE:
')
if [ -n "$found" ]; then
  echo "These lines cite a source file of ooRexx; state the behaviour instead:" >&2
  echo "$found" >&2
  exit 1
fi
