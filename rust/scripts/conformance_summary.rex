/* Render tracked inventories and optional gate observations as Markdown. */
numeric digits 18
signal on syntax name failed
parse source . . script
root = .file~new(script)~parentFile~parentFile~parent

options = .table~new
args = .syscargs~allItems
do i = 1 to args~items
  parse value args[i] with option '=' attached
  if option == '--help' | option == '-h' then do
    say 'Usage: rexx conformance_summary.rex [--corpus-log FILE] [--suite-report FILE] [--run-commit SHA]'
    exit 0
  end
  if wordpos(option, '--corpus-log --suite-report --run-commit') = 0 then
    call fail 'unknown option:' option, 2
  if pos('=', args[i]) = 0 then do
    i += 1
    if i > args~items then call fail 'missing value for' option, 2
    attached = args[i]
    if attached~startsWith('--') then call fail 'missing value for' option, 2
  end
  if attached == '' then call fail 'missing value for' option, 2
  options[option] = attached
end

/* Keep report paths relative to the caller, even when Git runs in the checkout. */
do option over .array~of('--corpus-log', '--suite-report')
  if options~hasIndex(option) then
    options[option] = .file~new(options[option])~absolutePath
end
if directory(root) == '' then call fail 'cannot enter the checkout:' root
commit = gitId('rev-parse HEAD')
oracle = gitId('rev-parse' commit':third_party/ooRexx')
if (options~hasIndex('--corpus-log') | options~hasIndex('--suite-report')) & ,
    options['--run-commit'] \== commit then
  call fail 'observations require --run-commit' commit

revisions = .array~new
do line over inventory(commit, 'suite-revision.txt')
  if \blank(line) then revisions~append(strip(line))
end
if revisions~items <> 1 then call fail 'suite-revision.txt does not contain an SVN revision'
revision = revisions[1]
if \decimal(revision) then call fail 'suite-revision.txt does not contain an SVN revision'
programs = dataLines(inventory(commit, 'differential.txt'))~items
unfiled = records(inventory(commit, 'unfiled.txt'), 2, 'unfiled.txt')
builtins = .table~new
do fields over records(inventory(commit, 'builtin-status.txt'), 2, 'builtin-status.txt')
  status = fields[2]
  if \builtins~hasIndex(status) then builtins[status] = .array~new
  builtins[status]~append(fields[1])
end
deviations = .table~new
do fields over records(inventory(commit, 'deviations.tsv'), 4, 'deviations.tsv')
  call increment deviations, fields[2]
end
counts = .table~new
reasons = .table~new
tab = '09'x
do name over .array~of('suite-licences.txt', 'suite-not-run.txt')
  lines = inventory(commit, name)
  call checkRevision lines, revision, name
  do fields over records(lines, 4, name)
    area = suiteArea(fields[1])
    category = fields[3]
    select
      when category == 'NotRun' then kind = 'Not run'
      when category == 'Unspecified' then kind = 'Assertion allowance'
      otherwise kind = 'Test licence'
    end
    if \counts~hasIndex(area) then counts[area] = .table~new
    call increment counts[area], kind
    call increment reasons, area || tab || category || tab || fields[4]
  end
end

/* Validate observations before writing any part of the summary. */
corpus = .nil
if options~hasIndex('--corpus-log') then corpus = corpusResults(readLines(options['--corpus-log']), programs)
suite = .nil
if options~hasIndex('--suite-report') then suite = suiteResults(readLines(options['--suite-report']), revision)

say '# Interrexx conformance'
say
say '- Interrexx inventory commit ' || code(commit)
say '- ooRexx pin ' || code(oracle) || ' from ' || code('third_party/ooRexx')
say '- ooTest pin r' || revision
say
say '## Tracked inventories'
say
say 'These counts describe recorded coverage and exclusions.'
say
say '-' programs 'programs are listed in ' || code('differential.txt') || '.'
say '-' unfiled~items 'programs are outside the differential in ' || code('unfiled.txt') || '.'
say '- ' || code('deviations.tsv') || ' lists' tally(deviations, 'active') ,
    'active and' tally(deviations, 'withdrawn') 'withdrawn Deviations.'
say
call tableHeader .array~of('Recorded built-in probe status', 'Names', 'Detail')
do status over builtins~allIndexes~sort
  detail = ''
  if status \== 'implemented' then detail = builtins[status]~makeString('L', ', ')
  call tableRow .array~of(status, builtins[status]~items, detail)
end
say
say 'The suite inventory groups ooTest paths by area. Test licences describe why a test'
say 'is outside the required pass set. Assertion allowances leave the test in that set.'
say code('suite-not-run.txt') 'records framework exclusions on the gate host.'
say
call tableHeader .array~of('ooTest area', 'Test licences', 'Assertion allowances', 'Framework exclusions')
do area over counts~allIndexes~sort
  row = counts[area]
  call tableRow .array~of(area, tally(row, 'Test licence'), tally(row, 'Assertion allowance'), tally(row, 'Not run'))
end
say
say '## Observed results'
say
say '### Differential corpus'
say
if corpus == .nil then say 'No strict corpus log supplied.'
else do
  say 'Strict differential reports cover' corpus[1] 'of' corpus[2] 'shards.'
  say 'Counts describe those shards only. Programs that did not finish are among the non-matches.'
  say
  call tableHeader .array~of('Program comparisons reported', 'Matching', 'Did not match')
  call tableRow .array~of(corpus[3], corpus[4], corpus[3] - corpus[4])
end
say
say '### ooTest suite'
say
if suite == .nil then say 'No ooTest per-test report supplied.'
else call printSuite suite, revision
say
say '## Recorded exclusions and allowances'
say
say '### ooTest'
say
call tableHeader .array~of('Area', 'Category', 'Reason', 'Tests')
do key over reasons~allIndexes~sort
  row = key~makeArray(tab)
  row~append(reasons[key])
  call tableRow row
end
say
say '### Corpus programs outside the differential'
say
call tableHeader .array~of('Program', 'Reason')
do row over unfiled
  call tableRow row
end
exit 0

failed:
  problem = condition('O')
  if problem~code \== '93.900' then raise propagate
  details = problem~additional
  if details~items <> 3 | details[1] \== 'conformance summary' then raise propagate
  .stderr~say(details[1] || ':' details[2])
  exit details[3]

::routine fail
  use strict arg message, status = 1
  raise syntax 93.900 array ('conformance summary', message, status)

::routine git
  use strict arg command
  output = .array~new
  errors = .array~new
  address path 'git' command with output using (output) error using (errors)
  if rc <> 0 then call fail 'git' command 'failed:' errors~makeString('L', ' ')
  return output

::routine inventory
  use strict arg commit, name
  return git('show' commit':rust/corpus/'name)

::routine gitId
  use strict arg command
  lines = git(command)
  if lines~items <> 1 then call fail 'git' command 'did not return one object ID'
  id = lines[1]
  if (length(id) <> 40 & length(id) <> 64) | verify(id, '0123456789abcdef') <> 0 then
    call fail 'git' command 'did not return a full object ID'
  return id

::routine readLines
  use strict arg path
  file = .stream~new(path)
  signal on notready name cannotRead
  if file~open('read') \== 'READY:' then signal cannotRead
  lines = .array~new
  do while file~lines('N') > 0
    lines~append(file~lineIn)
  end
  file~close
  return lines
cannotRead:
  call fail 'cannot read' path

::routine blank
  use strict arg value
  return strip(translate(value, ' ', '09'x)) == ''

::routine decimal
  use strict arg value
  return value \== '' & verify(value, '0123456789') = 0

::routine dataLines
  use strict arg lines
  data = .array~new
  do line over lines
    if \line~startsWith('#') & \blank(line) then data~append(line)
  end
  return data

::routine records
  use strict arg lines, width, source
  rows = .array~new
  do line over dataLines(lines)
    fields = line~makeArray('09'x)
    if fields~items <> width then call fail source 'has a malformed row:' line
    do field over fields
      if blank(field) then call fail source 'has a malformed row:' line
    end
    rows~append(fields)
  end
  return rows

::routine metadata
  use strict arg lines, prefix, source
  found = 0
  do line over lines
    if \line~startsWith(prefix) then iterate
    found += 1
    value = substr(line, length(prefix) + 1)
  end
  if found <> 1 then call fail source 'must contain one' strip(prefix) 'line'
  if value == '' then call fail source 'has empty metadata for' strip(prefix)
  return value

::routine checkRevision
  use strict arg lines, revision, source
  if metadata(lines, '# ootest revision ', source) \== revision then
    call fail source 'does not match the ooTest pin r' || revision

::routine increment
  use strict arg counts, key
  counts[key] = tally(counts, key) + 1

::routine tally
  use strict arg counts, key
  if counts~hasIndex(key) then return counts[key]
  return 0

::routine suiteArea
  use strict arg group
  group = translate(group)
  select
    when wordpos(group, 'BASE/BIF/CHARIN BASE/BIF/CHAROUT BASE/BIF/CHARS BASE/BIF/LINEIN BASE/BIF/LINEOUT BASE/BIF/LINES BASE/BIF/STREAM BASE/CLASS/STREAM') > 0 then return 'Streams'
    when wordpos(group, 'BASE/CLASS/ALARM BASE/CLASS/EVENTSEMAPHORE BASE/CLASS/MESSAGE BASE/CLASS/MUTEXSEMAPHORE BASE/CLASS/TICKER BASE/KEYWORD/GUARD BASE/KEYWORD/REPLY') > 0 then return 'Concurrency'
    when group~startsWith('API/') then return 'Native API'
    when group~startsWith('BASE/BIF/') then return 'Built-in functions'
    when group~startsWith('BASE/CLASS/') then return 'Classes'
    when group~startsWith('BASE/DIRECTIVES/') then return 'Directives'
    when group~startsWith('BASE/KEYWORD/') then return 'Keywords'
    when group~startsWith('BASE/REXXUTIL/') then return 'RexxUtil'
    when group~startsWith('EXTENSIONS/PLATFORM/') then return 'Platform extensions'
    when group~startsWith('EXTENSIONS/RXSOCK/') then return 'RxSock'
    otherwise return 'Other'
  end

::routine corpusResults
  use strict arg lines, programs
  prefix = 'interrexx differential corpus report -- rust/corpus/differential.txt -- shard '
  strict = 'mode: STRICT (the gate) -- REXX_CORPUS_GATE is set'
  seen = .table~new
  matched = 0
  total = 0
  width = 0
  do i = 1 to lines~items - 2
    if \lines[i]~startsWith(prefix) | lines[i + 1] \== strict then iterate
    parse value substr(lines[i], length(prefix) + 1) with shard ' of ' count
    parse value lines[i + 2] with matches ' of ' size ' matching' trailing
    if trailing \== '' | right(lines[i + 2], 9) \== ' matching' then iterate
    if \decimal(shard) | \decimal(count) | \decimal(matches) | \decimal(size) then iterate
    shard = shard + 0
    count = count + 0
    if seen~items = 0 then width = count
    if width < 1 | count <> width | shard >= width | seen~hasIndex(shard) then
      call fail 'corpus log has repeated or inconsistent shard indices'
    expected = 0
    if shard < programs then expected = 1 + (programs - shard - 1) % width
    if size <> expected | matches > size then
      call fail 'corpus shard' shard 'does not match the tracked inventory size'
    seen[shard] = .true
    matched += matches
    total += size
  end
  if seen~items = 0 then call fail 'corpus log has no strict differential report'
  return .array~of(seen~items, width, total, matched)

::routine suiteResults
  use strict arg lines, revision
  source = 'ooTest tests.tsv'
  tab = '09'x
  call checkRevision lines, revision, source
  selected = metadata(lines, '# groups ', source)
  lines = dataLines(lines)
  if lines~items = 0 then call fail source 'has no per-test header'
  if lines[1] \== .array~of('group', 'test', 'row', 'oracle', 'crate')~makeString('L', tab) then
    call fail source 'has no per-test header'
  lines~delete(1)
  counts = .table~new
  seen = .table~new
  valid = 'passed|failed|blocked|started|blocked at load|not started'~makeArray('|')
  engines = .array~of('ooRexx', 'interrexx')
  do fields over records(lines, 5, source)
    identity = fields[1] || tab || fields[2]
    if seen~hasIndex(identity) then call fail source 'repeats' fields[1] fields[2]
    seen[identity] = .true
    area = suiteArea(fields[1])
    do column = 4 to 5
      engine = engines[column - 3]
      parse value fields[column] with status ': ' .
      if \valid~hasItem(status) then call fail source 'has an unknown verdict:' fields[column]
      key = area || tab || engine
      if \counts~hasIndex(key) then counts[key] = .table~new
      call increment counts[key], status
    end
  end
  if seen~items = 0 then call fail source 'has no test results'
  return .array~of(selected, counts, valid)

::routine printSuite
  use strict arg suite, revision
  selected = suite[1]
  counts = suite[2]
  valid = suite[3]
  scope = 'all groups'
  if selected \== 'all' then scope = 'selected groups (' || escapeHtml(selected) || ')'
  say 'ooTest r' || revision || ', ' || scope || '. These are per-test outcomes as recorded by the suite gate.'
  say 'A test blocked at load never started. A started test can lack a final verdict.'
  say
  call tableHeader .array~of('Area', 'Engine', 'Started', 'Passed', 'Failed', 'Blocked in test', ,
    'Started without verdict', 'Blocked at load', 'Not started')
  do key over counts~allIndexes~sort
    outcomes = counts[key]
    row = key~makeArray('09'x)
    started = 0
    do i = 1 to 4
      started += tally(outcomes, valid[i])
    end
    row~append(started)
    do status over valid
      row~append(tally(outcomes, status))
    end
    call tableRow row
  end

::routine escapeHtml
  use strict arg value
  value = changestr('&', value, '&amp;')
  value = changestr('<', value, '&lt;')
  return changestr('>', value, '&gt;')

::routine code
  use strict arg value
  return '60'x || value || '60'x

::routine tableRow
  use strict arg cells
  columns = .array~new
  do cell over cells
    columns~append(changestr('|', escapeHtml(cell), '&#124;'))
  end
  say '| ' || columns~makeString('L', ' | ') || ' |'

::routine tableHeader
  use strict arg columns
  call tableRow columns
  say '| ' || copies('--- | ', columns~items - 1) || '--- |'
