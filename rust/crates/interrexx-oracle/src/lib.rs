//! Runs a Rexx program under an interpreter and captures its observable output.

mod normalize;
pub mod private_service;
pub use normalize::{Divergence, diff, normalize};

use std::path::{Path, PathBuf};
use std::process::Command;

/// Everything a Rexx program can be observed to produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

/// An interpreter under test, plus the loader paths it needs.
#[derive(Debug, Clone)]
pub struct Interpreter {
    pub binary: PathBuf,
    pub library_paths: Vec<PathBuf>,
}

impl Interpreter {
    pub fn run(&self, program: &Path, args: &[String], cwd: &Path) -> std::io::Result<Outcome> {
        self.run_with_api_services(program, args, cwd, &[])
    }

    /// Runs a program using the private API services owned by the caller.
    pub fn run_with_api_services(
        &self,
        program: &Path,
        args: &[String],
        cwd: &Path,
        services: &[&private_service::PrivateService],
    ) -> std::io::Result<Outcome> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg(program).args(args).current_dir(cwd);
        let joined = std::env::join_paths(&self.library_paths)
            .expect("library paths must not contain the path separator");
        for var in ["LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH"] {
            cmd.env(var, &joined);
        }
        for service in services {
            service.configure(&mut cmd);
        }
        let out = cmd.output()?;
        Ok(Outcome {
            stdout: out.stdout,
            stderr: out.stderr,
            exit_code: out.status.code().unwrap_or(-1),
        })
    }
}
