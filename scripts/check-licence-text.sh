#!/bin/sh
# Fails when a tracked text file names the licence of ooRexx, by its name or
# by its abbreviation. interrexx is MIT OR Apache-2.0 and is written in its own
# words, so no file of it may carry or mention that licence. Submodules are not
# searched.
#
# The pattern is split so that this script does not match itself.
#
# NOTICE is exempt: it attributes the error message texts that the build
# generates from ooRexx's message catalogue, and names ooRexx's licence for
# that attribution.
set -eu
cd "$(git rev-parse --show-toplevel)"
found=$(git ls-files -z -- . ':(exclude)NOTICE' | perl -0 -ne '
  chomp;
  next unless -f $_ && -T $_;
  open my $f, "<", $_ or die "$_: $!";
  local $/;
  my $text = <$f>;
  print "$_\n" if $text =~ /\bC[P]L\b|Common\s+Public\s+License/i;
')
if [ -n "$found" ]; then
  echo "These files name the ooRexx licence:" >&2
  echo "$found" >&2
  exit 1
fi
