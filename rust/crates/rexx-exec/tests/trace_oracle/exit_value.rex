/* EXIT <expr>'s own >>> value line.
 * No corpus program contains such a clause, so it is pinned here.
 *
 *
 * The adjacent passing case is deliberately NOT here: a *bare* EXIT traces
 * no value line at all, and call_arguments.rex, function_call.rex and
 * use_arg_alias.rex each reach one, so that half is already pinned by a
 * witness whose transcript would grow a line if this arm ever traced
 * unconditionally.
 *
 * The DO is there so the traced EXIT is not a top-level clause. It does not
 * pin the indent: Deviation 0 normalises the space run this file's own
 * comparison sees. */
trace r
say 'a'
do
  exit 1 + 1
end
