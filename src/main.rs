#[macro_use]
extern crate error_chain;
extern crate envoption;
extern crate nix;

use error_chain::ChainedError;
use nix::{fcntl, sched, unistd};
use std::ffi::CString;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::io::IntoRawFd;
use std::{env, fs, num, thread};

//XXX(lucab): should this be configurable?
const HOST_PROCFS: &'static str = "/proc";

error_chain!{
    foreign_links {
        Unix(nix::Error);
        Env(envoption::EnvOptionError<num::ParseIntError>);
    }
}

fn main() {
    if let Err(e) = run() {
        eprint!("{}", e.display_chain());
        std::process::exit(253);
    }
}

fn run() -> Result<()> {
    let mut argv = env::args_os();
    if argv.len() < 2 {
        // Non-interactive execution, park the process to hold the container.
        thread::park();
    }

    let target_pid: u32 = envoption::require("KUBOX_TARGET_PID")?;
    let bin = argv.nth(1).ok_or_else(|| "not enough arguments")?;
    let relbin = String::from("./") + &bin.to_string_lossy();

    // TODO(lucab): mark this CLOEXEC
    let basedir = fs::File::open("/").chain_err(|| "unable to open /")?;

    let targets = vec!["uts", "ipc", "cgroup", "net", "pid", "mnt", "user"];
    let mut fds = Vec::with_capacity(targets.len());
    for name in targets {
        let path = format!("{}/{}/ns/{}", HOST_PROCFS, target_pid, &name);
        let ns_fd = fs::File::open(&path).chain_err(
            || format!("unable to open {}", &path),
        )?;
        fds.push((name, ns_fd));
    }

    for (name, fd) in fds {
        let res = sched::setns(fd.into_raw_fd(), sched::CloneFlags::empty());
        if name == "user" {
            // setns may fail if current and target user namespaces are the same,
            // which is currently always the case in k8s, so just continue here.
            continue;
        }
        res.chain_err(|| format!("unable to setns for {}", &name))?;
    }

    unistd::execveat(
        basedir.into_raw_fd(),
        &CString::new(relbin).unwrap(),
        &env::args_os()
            .skip(1)
            .map(|s| CString::new(s.into_vec()).unwrap())
            .collect::<Vec<CString>>(),
        // TODO(lucab): forward environment to child.
        &[],
        fcntl::AtFlags::empty(),
    )?;
    Ok(())
}
