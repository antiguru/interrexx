use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A thread stack larger than any 64-bit host's address space, so a spawn
/// asking for it fails whether or not an address cap is enforced: macOS
/// enforces none, and has room for 64 TiB.
#[cfg(target_pointer_width = "64")]
pub(crate) const UNSPAWNABLE_STACK: usize = 1 << 62;

/// A thread stack the whole of a 32-bit address space but its last page, so a
/// spawn asking for it fails: the program's own mappings already sit in that
/// space.
#[cfg(not(target_pointer_width = "64"))]
pub(crate) const UNSPAWNABLE_STACK: usize = usize::MAX & !0xffff;

/// Whether the test driver explicitly selected ThreadSanitizer resources.
pub(crate) fn thread_sanitizer() -> bool {
    std::env::var("REXX_TEST_THREAD_SANITIZER").as_deref() == Ok("1")
}

/// Runs the named unit test alone with bounded virtual address space and an
/// open, empty stdin pipe. An explicitly instrumented test driver omits the
/// virtual-address limit. The resident cap is inherited from the test driver.
pub(crate) fn isolated(name: &str) -> bool {
    isolated_with_input(name, None)
}

#[expect(
    clippy::disallowed_methods,
    reason = "bounds the isolated test process using real time"
)]
pub(crate) fn isolated_with_input(name: &str, input: Option<&[u8]>) -> bool {
    const CHILD: &str = "REXX_POOL_FAILURE_CHILD";
    if std::env::var(CHILD).as_deref() == Ok(name) {
        let pool = crate::scheduler::Pool::new(UNSPAWNABLE_STACK, 1);
        match pool.reserve() {
            Err(_) => assert_eq!(pool.threads(), 0),
            Ok(Some(worker)) => {
                worker.run(Box::new(|| {}));
                panic!("huge-stack seam unexpectedly spawned a worker");
            }
            Ok(None) => panic!("huge-stack seam selected configured inline mode"),
        }
        return false;
    }
    let executable = std::env::current_exe().expect("test executable");
    // The sanitizer reserves its shadow mappings before the test body runs.
    // Only the explicit instrumented test driver may omit the address cap.
    let launch = if thread_sanitizer() {
        "import os,sys; os.execv(sys.argv[1],sys.argv[1:])"
    } else {
        // The cap is 8 GiB or the hard limit, where that is lower. A host that refuses an address
        // cap (macOS, which does not enforce one either) runs without it; Linux must take it.
        "import os,resource,sys\n\
         c=8<<30\n\
         h=resource.getrlimit(resource.RLIMIT_AS)[1]\n\
         c=c if h==resource.RLIM_INFINITY else min(c,h)\n\
         try:\n resource.setrlimit(resource.RLIMIT_AS,(c,c))\n\
         except (ValueError,OSError):\n\
         \x20if sys.platform.startswith('linux'): raise\n\
         os.execv(sys.argv[1],sys.argv[1:])"
    };
    let mut child = Command::new("python3")
        .args(["-c", launch])
        .arg(executable)
        .args(["--exact", name, "--nocapture"])
        .env(CHILD, name)
        .stdin(Stdio::piped())
        .spawn()
        .expect("isolated test");
    let mut writer = child.stdin.take().expect("open empty pipe");
    if let Some(input) = input {
        std::io::Write::write_all(&mut writer, input).expect("test input");
    }
    let deadline = Instant::now() + Duration::from_secs(25);
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait") {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().expect("kill timed out child");
            child.wait().expect("reap timed out child");
            panic!("isolated test blocked, possibly reading stdin inline");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    drop(writer);
    assert!(status.success(), "isolated test failed: {status}");
    true
}

/// Set in the process a test runs its case in alone ([`alone`]).
const ALONE: &str = "REXX_TEST_CASE_ALONE";

/// Whether this is the process the unit test `name`, its full path, runs its
/// case in. Elsewhere it runs this binary again for that test alone and
/// asserts that it passed, so that what a process keeps for the rest of its
/// life, such as a library mapped (`r[native.loading.static]`) or the signal
/// handlers an instance installs, is that case's alone under `cargo test`,
/// as it is under nextest.
pub(crate) fn alone(name: &str) -> bool {
    if std::env::var(ALONE).as_deref() == Ok(name) {
        return true;
    }
    let output = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", name, "--test-threads=1"])
        .env(ALONE, name)
        .stdin(Stdio::null())
        .output()
        .expect("the case's own process");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("1 passed"), "{stdout}");
    false
}
