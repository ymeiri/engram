#[cfg(not(all(
    target_os = "linux",
    target_arch = "aarch64",
    target_env = "musl",
    target_vendor = "unknown"
)))]
compile_error!("the entropy failure probe requires aarch64-unknown-linux-musl");

#[allow(dead_code)]
#[path = "../src/seccomp.rs"]
mod seccomp;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Component, Path, PathBuf};
use std::process;

const UUID_BYTES: usize = 36;
const MARKER_BYTE: u8 = b'M';

const CHILD_SETUP_FAILED: i32 = 70;
const CHILD_FILTER_FAILED: i32 = 71;
const CHILD_MARKER_FAILED: i32 = 72;
const CHILD_RESULT_FAILED: i32 = 73;

const NR_OPENAT: u32 = 56;
const NR_TKILL: u32 = 130;
const NR_GETRANDOM: u32 = 278;
const NR_OPENAT2: u32 = 437;

const BPF_LD_W_ABS: u16 = 0x20;
const BPF_JMP_JEQ_K: u16 = 0x15;
const BPF_RET_K: u16 = 0x06;

const SECCOMP_SET_MODE_FILTER: libc::c_ulong = 1;
const SECCOMP_RET_KILL_PROCESS: u32 = 0x8000_0000;
const SECCOMP_RET_TRAP: u32 = 0x0003_0000;
const SECCOMP_RET_ERRNO_EPERM: u32 = 0x0005_0001;
const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;

const OFFSET_NR: u32 = 0;
const OFFSET_ARCH: u32 = 4;

unsafe extern "C" {
    fn c2b2a_generate_unique_id(output: *mut u8) -> libc::c_int;
}

#[derive(Clone, Copy)]
enum Mode {
    SuccessfulProductionNoFile,
    FailureFallback,
    ProductionFailureComposition,
}

impl Mode {
    const ALL: [Self; 3] = [
        Self::SuccessfulProductionNoFile,
        Self::FailureFallback,
        Self::ProductionFailureComposition,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::SuccessfulProductionNoFile => "successful-production-no-file",
            Self::FailureFallback => "failure-fallback",
            Self::ProductionFailureComposition => "production-failure-composition",
        }
    }

    const fn installs_production_filters(self) -> bool {
        !matches!(self, Self::FailureFallback)
    }

    const fn expected(self) -> ExpectedOutcome {
        match self {
            Self::SuccessfulProductionNoFile => ExpectedOutcome::Exit(0),
            Self::FailureFallback => ExpectedOutcome::Signal(libc::SIGABRT),
            Self::ProductionFailureComposition => ExpectedOutcome::Signal(libc::SIGSYS),
        }
    }
}

#[derive(Clone, Copy)]
enum ExpectedOutcome {
    Exit(i32),
    Signal(i32),
}

#[repr(C)]
struct SockFprog {
    len: u16,
    filter: *mut seccomp::Instruction,
}

const fn statement(code: u16, k: u32) -> seccomp::Instruction {
    seccomp::Instruction {
        code,
        jt: 0,
        jf: 0,
        k,
    }
}

const fn jump_eq(k: u32, jt: u8, jf: u8) -> seccomp::Instruction {
    seccomp::Instruction {
        code: BPF_JMP_JEQ_K,
        jt,
        jf,
        k,
    }
}

const SUCCESS_INJECTOR: [seccomp::Instruction; 9] = [
    statement(BPF_LD_W_ABS, OFFSET_ARCH),
    jump_eq(seccomp::aarch64::AUDIT_ARCH, 1, 0),
    statement(BPF_RET_K, SECCOMP_RET_KILL_PROCESS),
    statement(BPF_LD_W_ABS, OFFSET_NR),
    jump_eq(NR_OPENAT, 0, 1),
    statement(BPF_RET_K, SECCOMP_RET_TRAP),
    jump_eq(NR_OPENAT2, 0, 1),
    statement(BPF_RET_K, SECCOMP_RET_TRAP),
    statement(BPF_RET_K, SECCOMP_RET_ALLOW),
];

const FAILURE_FALLBACK_INJECTOR: [seccomp::Instruction; 13] = [
    statement(BPF_LD_W_ABS, OFFSET_ARCH),
    jump_eq(seccomp::aarch64::AUDIT_ARCH, 1, 0),
    statement(BPF_RET_K, SECCOMP_RET_KILL_PROCESS),
    statement(BPF_LD_W_ABS, OFFSET_NR),
    jump_eq(NR_OPENAT, 0, 1),
    statement(BPF_RET_K, SECCOMP_RET_TRAP),
    jump_eq(NR_TKILL, 0, 1),
    statement(BPF_RET_K, SECCOMP_RET_ALLOW),
    jump_eq(NR_GETRANDOM, 0, 1),
    statement(BPF_RET_K, SECCOMP_RET_ERRNO_EPERM),
    jump_eq(NR_OPENAT2, 0, 1),
    statement(BPF_RET_K, SECCOMP_RET_TRAP),
    statement(BPF_RET_K, SECCOMP_RET_ALLOW),
];

const FAILURE_COMPOSITION_INJECTOR: [seccomp::Instruction; 11] = [
    statement(BPF_LD_W_ABS, OFFSET_ARCH),
    jump_eq(seccomp::aarch64::AUDIT_ARCH, 1, 0),
    statement(BPF_RET_K, SECCOMP_RET_KILL_PROCESS),
    statement(BPF_LD_W_ABS, OFFSET_NR),
    jump_eq(NR_OPENAT, 0, 1),
    statement(BPF_RET_K, SECCOMP_RET_TRAP),
    jump_eq(NR_GETRANDOM, 0, 1),
    statement(BPF_RET_K, SECCOMP_RET_ERRNO_EPERM),
    jump_eq(NR_OPENAT2, 0, 1),
    statement(BPF_RET_K, SECCOMP_RET_TRAP),
    statement(BPF_RET_K, SECCOMP_RET_ALLOW),
];

fn injector(mode: Mode) -> &'static [seccomp::Instruction] {
    match mode {
        Mode::SuccessfulProductionNoFile => &SUCCESS_INJECTOR,
        Mode::FailureFallback => &FAILURE_FALLBACK_INJECTOR,
        Mode::ProductionFailureComposition => &FAILURE_COMPOSITION_INJECTOR,
    }
}

fn install_injector(mode: Mode) -> Result<(), ()> {
    let instructions = injector(mode);
    let len = u16::try_from(instructions.len()).map_err(|_| ())?;
    let mut program = SockFprog {
        len,
        filter: instructions.as_ptr().cast_mut(),
    };
    // SAFETY: the static instruction array and stack descriptor remain live while the kernel
    // copies the classic-BPF program. The exact syscall number is the frozen AArch64 value.
    let result = unsafe {
        libc::syscall(
            seccomp::aarch64::NR_SECCOMP as libc::c_long,
            SECCOMP_SET_MODE_FILTER,
            0 as libc::c_ulong,
            &mut program as *mut SockFprog,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(())
    }
}

fn prepare_child_state() -> Result<(), ()> {
    // SAFETY: every pointer passed to libc names a live object of the corresponding libc type.
    unsafe {
        for signal in [libc::SIGABRT, libc::SIGSYS] {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = libc::SIG_DFL;
            if libc::sigemptyset(&mut action.sa_mask) != 0
                || libc::sigaction(signal, &action, std::ptr::null_mut()) != 0
            {
                return Err(());
            }
        }

        let mut signals: libc::sigset_t = std::mem::zeroed();
        if libc::sigemptyset(&mut signals) != 0
            || libc::sigaddset(&mut signals, libc::SIGABRT) != 0
            || libc::sigaddset(&mut signals, libc::SIGSYS) != 0
            || libc::sigprocmask(libc::SIG_UNBLOCK, &signals, std::ptr::null_mut()) != 0
        {
            return Err(());
        }

        let mut pending: libc::sigset_t = std::mem::zeroed();
        if libc::sigpending(&mut pending) != 0
            || libc::sigismember(&pending, libc::SIGABRT) != 0
            || libc::sigismember(&pending, libc::SIGSYS) != 0
        {
            return Err(());
        }

        let zero_core = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if libc::setrlimit(libc::RLIMIT_CORE, &zero_core) != 0 {
            return Err(());
        }
        let mut observed_core: libc::rlimit = std::mem::zeroed();
        if libc::getrlimit(libc::RLIMIT_CORE, &mut observed_core) != 0
            || observed_core.rlim_cur != 0
            || observed_core.rlim_max != 0
        {
            return Err(());
        }

        if libc::prctl(
            libc::PR_SET_DUMPABLE,
            0 as libc::c_ulong,
            0 as libc::c_ulong,
            0 as libc::c_ulong,
            0 as libc::c_ulong,
        ) != 0
            || libc::prctl(
                libc::PR_GET_DUMPABLE,
                0 as libc::c_ulong,
                0 as libc::c_ulong,
                0 as libc::c_ulong,
                0 as libc::c_ulong,
            ) != 0
            || libc::prctl(
                libc::PR_SET_NO_NEW_PRIVS,
                1 as libc::c_ulong,
                0 as libc::c_ulong,
                0 as libc::c_ulong,
                0 as libc::c_ulong,
            ) != 0
            || libc::prctl(
                libc::PR_GET_NO_NEW_PRIVS,
                0 as libc::c_ulong,
                0 as libc::c_ulong,
                0 as libc::c_ulong,
                0 as libc::c_ulong,
            ) != 1
        {
            return Err(());
        }
    }
    Ok(())
}

fn production_programs() -> Result<(seccomp::Program, seccomp::Program), ()> {
    let bootstrap = seccomp::bootstrap_program().map_err(|_| ())?;
    let steady = seccomp::steady_program().map_err(|_| ())?;
    Ok((bootstrap, steady))
}

fn valid_uuid(bytes: &[u8; UUID_BYTES]) -> bool {
    for (index, byte) in bytes.iter().copied().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            if byte != b'-' {
                return false;
            }
        } else if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte) {
            return false;
        }
    }
    bytes[14] == b'4' && matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
}

fn child_main(mode: Mode, marker_fd: libc::c_int) -> ! {
    if prepare_child_state().is_err() {
        // SAFETY: `_exit` terminates only this post-fork child without running shared state.
        unsafe { libc::_exit(CHILD_SETUP_FAILED) }
    }

    let production = if mode.installs_production_filters() {
        match production_programs() {
            Ok(programs) => Some(programs),
            Err(()) => unsafe { libc::_exit(CHILD_FILTER_FAILED) },
        }
    } else {
        None
    };

    if install_injector(mode).is_err() {
        unsafe { libc::_exit(CHILD_FILTER_FAILED) }
    }
    if let Some((bootstrap, steady)) = production {
        if seccomp::install_program(&bootstrap).is_err()
            || seccomp::install_program(&steady).is_err()
        {
            unsafe { libc::_exit(CHILD_FILTER_FAILED) }
        }
    }

    let mut output = [0_u8; UUID_BYTES];
    let marker = [MARKER_BYTE];
    // SAFETY: `marker_fd` is this child's open write end and `marker` is one live byte.
    if unsafe { libc::write(marker_fd, marker.as_ptr().cast(), marker.len()) } != 1 {
        unsafe { libc::_exit(CHILD_MARKER_FAILED) }
    }

    // SAFETY: the C wrapper receives the fixed writable 36-byte buffer required by its ABI.
    let status = unsafe { c2b2a_generate_unique_id(output.as_mut_ptr()) };
    match mode {
        Mode::SuccessfulProductionNoFile if status == 0 && valid_uuid(&output) => {
            unsafe { libc::_exit(0) }
        }
        Mode::SuccessfulProductionNoFile
        | Mode::FailureFallback
        | Mode::ProductionFailureComposition => unsafe { libc::_exit(CHILD_RESULT_FAILED) },
    }
}

fn read_marker(fd: libc::c_int) -> bool {
    let mut byte = 0_u8;
    loop {
        // SAFETY: `fd` is the parent's live read end and `byte` is writable for one byte.
        let result = unsafe { libc::read(fd, (&mut byte as *mut u8).cast(), 1) };
        if result == 1 {
            return byte == MARKER_BYTE;
        }
        if result == 0 {
            return false;
        }
        if io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
            return false;
        }
    }
}

fn wait_for_child(pid: libc::pid_t) -> Option<libc::c_int> {
    let mut status = 0;
    loop {
        // SAFETY: `pid` identifies the one child and `status` is a live output integer.
        let result = unsafe {
            libc::waitpid(
                pid,
                &mut status,
                libc::WUNTRACED | libc::WCONTINUED,
            )
        };
        if result == pid {
            if libc::WIFEXITED(status) || libc::WIFSIGNALED(status) {
                return Some(status);
            }
            if libc::WIFSTOPPED(status) || libc::WIFCONTINUED(status) {
                // SAFETY: a nonterminal state was observed for this live child. SIGKILL cannot be
                // blocked, and the following wait reaps exactly this child before failure returns.
                unsafe {
                    libc::kill(pid, libc::SIGKILL);
                }
                loop {
                    let cleanup = unsafe { libc::waitpid(pid, &mut status, 0) };
                    if cleanup == pid {
                        break;
                    }
                    if cleanup == -1
                        && io::Error::last_os_error().raw_os_error() == Some(libc::EINTR)
                    {
                        continue;
                    }
                    break;
                }
                return None;
            }
            return None;
        }
        if result == -1 && io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        return None;
    }
}

fn outcome_matches(expected: ExpectedOutcome, status: libc::c_int) -> bool {
    if status & 0x80 != 0 {
        return false;
    }
    match expected {
        ExpectedOutcome::Exit(code) => libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == code,
        ExpectedOutcome::Signal(signal) => {
            libc::WIFSIGNALED(status) && libc::WTERMSIG(status) == signal
        }
    }
}

fn run_mode(mode: Mode) -> bool {
    let mut pipe_fds = [-1; 2];
    // SAFETY: `pipe_fds` supplies space for exactly two descriptors.
    if unsafe { libc::pipe2(pipe_fds.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
        return false;
    }

    // SAFETY: the parent is single-threaded and performs only async-signal-safe operations in the
    // child until returning to Rust code whose state was already present at the fork boundary.
    let pid = unsafe { libc::fork() };
    if pid == -1 {
        unsafe {
            libc::close(pipe_fds[0]);
            libc::close(pipe_fds[1]);
        }
        return false;
    }
    if pid == 0 {
        unsafe {
            libc::close(pipe_fds[0]);
        }
        child_main(mode, pipe_fds[1]);
    }

    unsafe {
        libc::close(pipe_fds[1]);
    }
    let status = wait_for_child(pid);
    let marker_valid = read_marker(pipe_fds[0]);

    let mut trailing = 0_u8;
    // SAFETY: after `waitpid`, the child has closed its only write end; EOF is required.
    let marker_eof = unsafe { libc::read(pipe_fds[0], (&mut trailing as *mut u8).cast(), 1) } == 0;
    unsafe {
        libc::close(pipe_fds[0]);
    }

    marker_valid
        && marker_eof
        && status.is_some_and(|observed| outcome_matches(mode.expected(), observed))
}

fn parse_probe_directory() -> Result<PathBuf, ()> {
    let mut arguments = env::args_os();
    let _program = arguments.next().ok_or(())?;
    let raw = arguments.next().ok_or(())?;
    if arguments.next().is_some() {
        return Err(());
    }
    validate_absolute_path(&raw)
}

fn validate_absolute_path(raw: &OsString) -> Result<PathBuf, ()> {
    let path = PathBuf::from(raw.as_os_str());
    if !path.is_absolute() {
        return Err(());
    }
    for component in path.components() {
        if !matches!(component, Component::RootDir | Component::Normal(_)) {
            return Err(());
        }
    }
    if path.file_name().is_none() {
        return Err(());
    }
    Ok(path)
}

fn create_probe_directory(path: &Path) -> Result<(), ()> {
    // SAFETY: this single-threaded parent intentionally fixes its process umask before creation.
    unsafe {
        libc::umask(0o077);
    }
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700).create(path).map_err(|_| ())?;

    let metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    if !metadata.file_type().is_dir()
        || metadata.mode() & 0o7777 != 0o700
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.gid() != unsafe { libc::getegid() }
        || fs::read_dir(path).map_err(|_| ())?.next().is_some()
    {
        return Err(());
    }
    env::set_current_dir(path).map_err(|_| ())
}

fn fail(message: &str, code: i32) -> ! {
    eprintln!("c2b2a-entropy-failure-probe: {message}");
    process::exit(code);
}

fn main() {
    let probe_directory = parse_probe_directory().unwrap_or_else(|()| {
        fail("expected one absolute fresh probe-directory path", 64);
    });
    if create_probe_directory(&probe_directory).is_err() {
        fail("could not create the fresh owner-only probe directory", 65);
    }

    for mode in Mode::ALL {
        if !run_mode(mode) {
            fail(mode.name(), 66);
        }
        println!("mode={} valid=true", mode.name());
    }

    let directory_changed = match fs::read_dir(&probe_directory) {
        Ok(mut entries) => entries.next().is_some(),
        Err(_) => true,
    };
    if directory_changed {
        fail("probe directory changed", 67);
    }
    println!("probe_valid=true");
}
