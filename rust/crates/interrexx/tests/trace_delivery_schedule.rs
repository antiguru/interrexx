//! A trace line's delivery to a short Rexx destination lets no other
//! activity's lines in, and one to a guarded destination that runs nested
//! code delivers every line, under any schedule of the simulation mode. The oracle
//! gives up the kernel only in an activation past `yieldInstructions` or at a
//! native call, and this destination's `lineOut` has neither.

use std::process::Command;

/// ooTest `base/keyword/TRACE` `test_forwarded3`, its `ArrayStream`
/// destination included: a REPLY continuation's `>I>` and `<I<` race the
/// replier's own exit lines.
const PROGRAM: &str = "as = .ArrayStream~new\n\
     .TraceOutput~destination(as)\n\
     src = .array~of('t=.test~new', 't~one', 'call syssleep 0.001', 'res=t~three', \
     'call syssleep 0.001', 'return res', '::class test', '::method one', \
     '  forward message \"onetwo\"', '  i=i/0', '::method onetwo', '  reply', \
     '::method three', '  reply 42', '  forward message \"threefour\" continue', \
     '  if result<>42 then i=i/0', '::method threefour', '  return 42', \
     '::options trace labels')\n\
     r = .routine~new('fw3', src)\n\
     res = r~call\n\
     .TraceOutput~destination\n\
     say 'res' res\n\
     do v over as\n\
     \x20 f = v['STACKFRAME']\n\
     \x20 if f~type == 'METHOD' then do\n\
     \x20   text = v['TRACELINE']\n\
     \x20   say left(text, verify(text, ' ') + 2) f~name f~executable~scope~id\n\
     \x20 end\n\
     end\n\
     ::class ArrayStream subclass Array inherit Stream\n\
     ::method init\n  expose line\n  line = 0\n\
     ::method lineOut\n  use arg object\n  parse arg line\n  self~append(object)\n  return 0\n";

/// What the oracle printed, 8 runs of 8: for each line of a method, kept by
/// the kind of its level, the indentation and flag of the announcement, the
/// method and its scope; the wording of the announcement's text is not
/// specified (`r[conv.error.text]`), and the routine's own lines are left out.
const ORACLE_STDOUT: &str = "res 42\n\
     \x20      >I> ONE TEST\n\
     \x20      >I> ONETWO TEST\n\
     \x20      <I< ONETWO TEST\n\
     \x20      <I< ONE TEST\n\
     \x20      >I> ONETWO TEST\n\
     \x20      <I< ONETWO TEST\n\
     \x20      >I> THREE TEST\n\
     \x20      <I< THREE TEST\n\
     \x20      >I> THREE TEST\n\
     \x20      >I> THREEFOUR TEST\n\
     \x20      <I< THREEFOUR TEST\n\
     \x20      <I< THREE TEST\n";

#[test]
fn a_short_destination_keeps_each_delivery_whole_under_every_schedule() {
    let dir = std::env::temp_dir().join(format!("rexx-trace-delivery-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("p.rex"), PROGRAM).expect("the program written");
    let mut differing = Vec::new();
    for seed in 1..=20 {
        for policy in ["uniform:1", "uniform:0.2", "pct:3,k=4"] {
            let mode = format!("sim:{seed},{policy}");
            let output = Command::new(env!("CARGO_BIN_EXE_rexx"))
                .arg(dir.join("p.rex"))
                .env("REXX_SWITCH_MODE", &mode)
                .stdin(std::process::Stdio::null())
                .output()
                .expect("rexx runs");
            if !output.status.success() || output.stdout != ORACLE_STDOUT.as_bytes() {
                differing.push(format!(
                    "{mode}:\n{}",
                    String::from_utf8_lossy(&output.stdout)
                ));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(differing.is_empty(), "{}", differing.join("\n"));
}

/// Two activities trace into one guarded destination whose `LINEOUT` runs
/// more than a short destination's clauses inside a method that `run` runs.
/// The delivery holds the guard under that nested construct, so it lets no
/// other activity run on top of it: one whose own line waited for that guard
/// would wait beneath itself. Every line of both is delivered under every
/// schedule, as the oracle delivers them.
#[test]
fn a_nested_guarded_destination_delivers_every_line_of_two_activities_under_every_schedule() {
    const NESTED: &str = "s = .sink~new\n\
         .traceoutput~destination(s)\n\
         a = .w~new~start('go', 'A')\n\
         b = .w~new~start('go', 'B')\n\
         say a~result b~result\n\
         .traceoutput~destination(.output)\n\
         say 'lines' s~lines\n\
         ::class w\n\
         ::method go\n\
         \x20 use arg n\n\
         \x20 trace r\n\
         \x20 x = n\n\
         \x20 y = n\n\
         \x20 trace off\n\
         \x20 return n\n\
         ::class sink\n\
         ::method init\n\
         \x20 expose count\n\
         \x20 count = 0\n\
         ::method lineout guarded\n\
         \x20 expose count\n\
         \x20 count += 1\n\
         \x20 return self~run(.method~new('w', 'do i = 1 to 200; end; return 0'))\n\
         ::method lines unguarded\n\
         \x20 expose count\n\
         \x20 return count\n";
    let dir = std::env::temp_dir().join(format!("rexx-trace-nested-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("p.rex"), NESTED).expect("the program written");
    let mut differing = Vec::new();
    for seed in 1..=20 {
        for policy in ["uniform:1", "uniform:0.2", "pct:3,k=4"] {
            let mode = format!("sim:{seed},{policy}");
            let output = Command::new(env!("CARGO_BIN_EXE_rexx"))
                .arg(dir.join("p.rex"))
                .env("REXX_SWITCH_MODE", &mode)
                .stdin(std::process::Stdio::null())
                .output()
                .expect("rexx runs");
            if !output.status.success() || output.stdout != b"A B\nlines 10\n" {
                differing.push(format!(
                    "{mode}: {:?}\n{}{}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(differing.is_empty(), "{}", differing.join("\n"));
}
