//! Shared owned-process lifecycle for strict native evaluation foundations.
//!
//! Spawning stays inside this module so every production child begins as the leader of an
//! evaluator-owned process group. The guard keeps the direct child WNOWAIT-reserved until stable
//! descendant absence is proven and retains the caller's original terminal deadline for unwind
//! cleanup.

#[cfg(test)]
use std::cell::Cell;
#[cfg(target_os = "linux")]
use std::fs;
use std::io::{Read, Write};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

#[cfg(test)]
thread_local! {
    static INJECT_NEXT_POST_SPAWN_CLEANUP_UNPROVEN: Cell<bool> = const { Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn inject_next_post_spawn_cleanup_unproven_for_test() {
    INJECT_NEXT_POST_SPAWN_CLEANUP_UNPROVEN.with(|fault| fault.set(true));
}

#[derive(Debug, Error)]
pub enum NativeExecutionError {
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid native execution data: {0}")]
    Invalid(String),
    #[error(
        "native execution process ownership changed from evaluator PID {owner_pid} to {observed_pid}"
    )]
    OwnerProcessMismatch { owner_pid: u32, observed_pid: u32 },
}

pub(crate) type NativeExecutionResult<T> = Result<T, NativeExecutionError>;

/// Spawn never collapses a post-spawn cleanup uncertainty into an ordinary error. Callers must
/// never classify the unproven variant as proven terminal; it retains its owner-bound guard while
/// the caller may durably freeze an ambiguous, permanently non-replayable outcome.
#[derive(Debug)]
#[must_use = "native execution spawn outcomes must be classified before terminalization"]
pub(crate) enum NativeExecutionSpawnOutcome {
    Spawned(NativeExecutionChildGuard),
    NotSpawned(NativeExecutionError),
    PostSpawnTerminal {
        process_id: u32,
        status: ExitStatus,
        error: NativeExecutionError,
    },
    PostSpawnCleanupUnproven {
        process_id: u32,
        error: NativeExecutionError,
        quarantine: NativeExecutionChildGuard,
    },
}

#[derive(Debug)]
#[must_use = "cleanup must be proven terminal or treated as unproven"]
pub(crate) enum NativeExecutionCleanupOutcome {
    ProvenTerminal(ExitStatus),
    CleanupUnproven(NativeExecutionError),
}

#[derive(Debug)]
pub(crate) struct NativeExecutionChildStdin {
    inner: ChildStdin,
    owner_pid: u32,
}

impl NativeExecutionChildStdin {
    fn require_owner(&self) -> NativeExecutionResult<()> {
        require_owner_pid(self.owner_pid)
    }

    #[cfg(unix)]
    pub(crate) fn configure_nonblocking(&self) -> std::io::Result<()> {
        use std::os::fd::AsRawFd;

        self.require_owner().map_err(pipe_owner_io_error)?;
        let descriptor = self.inner.as_raw_fd();
        // SAFETY: the descriptor remains owned by `inner`; F_GETFL only reads its flags.
        let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
        if flags < 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: the descriptor remains owned by `inner`; all existing flags are preserved.
        if unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    #[cfg(unix)]
    pub(crate) fn poll_writable(&self, timeout_ms: i32) -> std::io::Result<Option<i16>> {
        use std::os::fd::AsRawFd;

        self.require_owner().map_err(pipe_owner_io_error)?;
        let mut descriptor = libc::pollfd {
            fd: self.inner.as_raw_fd(),
            events: libc::POLLOUT,
            revents: 0,
        };
        // SAFETY: `descriptor` points to one live pollfd for this owner-bound stdin descriptor.
        let observed = unsafe { libc::poll(&mut descriptor, 1, timeout_ms) };
        if observed < 0 {
            return Err(std::io::Error::last_os_error());
        }
        if observed == 0 {
            return Ok(None);
        }
        Ok(Some(descriptor.revents))
    }
}

impl Write for NativeExecutionChildStdin {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.require_owner().map_err(pipe_owner_io_error)?;
        self.inner.write(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.require_owner().map_err(pipe_owner_io_error)?;
        self.inner.flush()
    }

    fn write_vectored(&mut self, buffers: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.require_owner().map_err(pipe_owner_io_error)?;
        self.inner.write_vectored(buffers)
    }
}

#[derive(Debug)]
pub(crate) struct NativeExecutionChildStdout {
    inner: ChildStdout,
    owner_pid: u32,
}

impl Read for NativeExecutionChildStdout {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        require_owner_pid(self.owner_pid).map_err(pipe_owner_io_error)?;
        self.inner.read(bytes)
    }

    fn read_vectored(&mut self, buffers: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        require_owner_pid(self.owner_pid).map_err(pipe_owner_io_error)?;
        self.inner.read_vectored(buffers)
    }
}

#[derive(Debug)]
pub(crate) struct NativeExecutionChildStderr {
    inner: ChildStderr,
    owner_pid: u32,
}

impl Read for NativeExecutionChildStderr {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        require_owner_pid(self.owner_pid).map_err(pipe_owner_io_error)?;
        self.inner.read(bytes)
    }

    fn read_vectored(&mut self, buffers: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        require_owner_pid(self.owner_pid).map_err(pipe_owner_io_error)?;
        self.inner.read_vectored(buffers)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeExecutionGroupAuthority {
    VerifiedLeader,
    ConstructedUnverified,
    UnownedUnverified,
}

#[derive(Debug)]
pub(crate) struct NativeExecutionChildGuard {
    child: Option<Child>,
    process_group_id: Option<i32>,
    group_authority: NativeExecutionGroupAuthority,
    owner_pid: u32,
    terminal_deadline: Instant,
    #[cfg(test)]
    cleanup_fault: Option<NativeExecutionCleanupFault>,
    #[cfg(test)]
    signal_attempt_count: Cell<u32>,
    #[cfg(test)]
    force_initial_observation_deadline_interrupt: bool,
    #[cfg(test)]
    fail_next_cleanup_attempt: bool,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeExecutionCleanupFault {
    PersistentEnumeration,
    PersistentSignalEperm,
}

impl NativeExecutionChildGuard {
    /// Spawns one child as the leader of a fresh process group and verifies the kernel-observed
    /// group identity before returning ownership to the caller.
    #[cfg(unix)]
    pub(crate) fn spawn(
        mut command: Command,
        terminal_deadline: Instant,
    ) -> NativeExecutionSpawnOutcome {
        use std::os::unix::process::CommandExt;

        if Instant::now() >= terminal_deadline {
            return NativeExecutionSpawnOutcome::NotSpawned(NativeExecutionError::Invalid(
                "refusing to spawn after the absolute terminal deadline".into(),
            ));
        }
        command.process_group(0);
        let child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                return NativeExecutionSpawnOutcome::NotSpawned(error.into());
            }
        };
        Self::from_spawned_process_group(child, terminal_deadline, true)
    }

    #[cfg(not(unix))]
    pub(crate) fn spawn(
        _command: Command,
        _terminal_deadline: Instant,
    ) -> NativeExecutionSpawnOutcome {
        NativeExecutionSpawnOutcome::NotSpawned(NativeExecutionError::Invalid(
            "owned process-group execution requires a supported Unix host".into(),
        ))
    }

    /// Tests may adopt a child they explicitly created as a process-group leader. Production code
    /// cannot bypass this module's spawn-and-verify boundary.
    #[cfg(all(test, unix))]
    pub(crate) fn adopt_spawned_group_for_test(
        child: Child,
        terminal_deadline: Instant,
    ) -> NativeExecutionResult<Self> {
        match Self::from_spawned_process_group(child, terminal_deadline, false) {
            NativeExecutionSpawnOutcome::Spawned(guard) => Ok(guard),
            NativeExecutionSpawnOutcome::NotSpawned(error)
            | NativeExecutionSpawnOutcome::PostSpawnTerminal { error, .. }
            | NativeExecutionSpawnOutcome::PostSpawnCleanupUnproven { error, .. } => Err(error),
        }
    }

    #[cfg(unix)]
    fn from_spawned_process_group(
        child: Child,
        terminal_deadline: Instant,
        group_owned_by_construction: bool,
    ) -> NativeExecutionSpawnOutcome {
        let process_id = child.id();
        let process_group_id = i32::try_from(process_id).ok();
        let mut guard = Self {
            child: Some(child),
            process_group_id,
            group_authority: if group_owned_by_construction {
                NativeExecutionGroupAuthority::ConstructedUnverified
            } else {
                NativeExecutionGroupAuthority::UnownedUnverified
            },
            owner_pid: std::process::id(),
            terminal_deadline,
            #[cfg(test)]
            cleanup_fault: None,
            #[cfg(test)]
            signal_attempt_count: Cell::new(0),
            #[cfg(test)]
            force_initial_observation_deadline_interrupt: false,
            #[cfg(test)]
            fail_next_cleanup_attempt: false,
        };
        let Some(process_group_id) = process_group_id else {
            return guard.reject_after_spawn(NativeExecutionError::Invalid(
                "direct child PID exceeded the process-group boundary".into(),
            ));
        };
        // SAFETY: getpgid is a read-only kernel query for the live direct child just spawned by
        // this module (or explicitly supplied by a test-only adoption path).
        let observed_group = unsafe { libc::getpgid(process_group_id) };
        if observed_group != process_group_id {
            let detail = if observed_group < 0 {
                format!(
                    "cannot verify evaluator-owned process-group identity: {}",
                    std::io::Error::last_os_error()
                )
            } else {
                "direct child was not the leader of its own process group".to_string()
            };
            return guard.reject_after_spawn(NativeExecutionError::Invalid(detail));
        }
        guard.group_authority = NativeExecutionGroupAuthority::VerifiedLeader;
        #[cfg(test)]
        if group_owned_by_construction
            && INJECT_NEXT_POST_SPAWN_CLEANUP_UNPROVEN.with(|fault| fault.replace(false))
        {
            guard.fail_next_cleanup_attempt = true;
            return guard.reject_after_spawn(NativeExecutionError::Invalid(
                "injected post-spawn cleanup uncertainty".into(),
            ));
        }
        if Instant::now() >= terminal_deadline {
            return guard.reject_after_spawn(NativeExecutionError::Invalid(
                "absolute terminal deadline elapsed while verifying the spawned process group"
                    .into(),
            ));
        }
        NativeExecutionSpawnOutcome::Spawned(guard)
    }

    fn reject_after_spawn(mut self, primary: NativeExecutionError) -> NativeExecutionSpawnOutcome {
        let process_id = self
            .child
            .as_ref()
            .expect("post-spawn rejection retains its child")
            .id();
        match self.terminate_group_and_reap_with_status_until(self.terminal_deadline) {
            NativeExecutionCleanupOutcome::ProvenTerminal(status) => {
                NativeExecutionSpawnOutcome::PostSpawnTerminal {
                    process_id,
                    status,
                    error: primary,
                }
            }
            NativeExecutionCleanupOutcome::CleanupUnproven(cleanup_error) => {
                NativeExecutionSpawnOutcome::PostSpawnCleanupUnproven {
                    process_id,
                    error: NativeExecutionError::Invalid(format!(
                        "{primary}; post-spawn cleanup remained unproven: {cleanup_error}"
                    )),
                    quarantine: self,
                }
            }
        }
    }

    fn require_owner(&self) -> NativeExecutionResult<()> {
        require_owner_pid(self.owner_pid)
    }

    #[cfg(test)]
    pub(crate) fn inject_cleanup_fault(&mut self, fault: NativeExecutionCleanupFault) {
        self.cleanup_fault = Some(fault);
    }

    #[cfg(test)]
    pub(crate) fn inject_next_cleanup_attempt_failure_for_test(&mut self) {
        self.fail_next_cleanup_attempt = true;
    }

    #[cfg(test)]
    fn inject_initial_observation_deadline_interrupt(&mut self) {
        self.force_initial_observation_deadline_interrupt = true;
    }

    pub(crate) fn take_stdin(
        &mut self,
    ) -> NativeExecutionResult<Option<NativeExecutionChildStdin>> {
        self.require_owner()?;
        Ok(self
            .child
            .as_mut()
            .expect("child guard is armed")
            .stdin
            .take()
            .map(|inner| NativeExecutionChildStdin {
                inner,
                owner_pid: self.owner_pid,
            }))
    }

    pub(crate) fn take_stdout(
        &mut self,
    ) -> NativeExecutionResult<Option<NativeExecutionChildStdout>> {
        self.require_owner()?;
        Ok(self
            .child
            .as_mut()
            .expect("child guard is armed")
            .stdout
            .take()
            .map(|inner| NativeExecutionChildStdout {
                inner,
                owner_pid: self.owner_pid,
            }))
    }

    pub(crate) fn take_stderr(
        &mut self,
    ) -> NativeExecutionResult<Option<NativeExecutionChildStderr>> {
        self.require_owner()?;
        Ok(self
            .child
            .as_mut()
            .expect("child guard is armed")
            .stderr
            .take()
            .map(|inner| NativeExecutionChildStderr {
                inner,
                owner_pid: self.owner_pid,
            }))
    }

    #[cfg(test)]
    pub(crate) fn try_wait_for_test(&mut self) -> NativeExecutionResult<Option<ExitStatus>> {
        self.require_owner()?;
        Ok(self
            .child
            .as_mut()
            .expect("child guard is armed")
            .try_wait()?)
    }

    pub(crate) fn process_id(&self) -> NativeExecutionResult<u32> {
        self.require_owner()?;
        Ok(self.child.as_ref().expect("child guard is armed").id())
    }

    pub(crate) fn is_armed(&self) -> bool {
        self.child.is_some()
    }

    #[cfg(unix)]
    pub(crate) fn exited_without_reap(&mut self, deadline: Instant) -> NativeExecutionResult<bool> {
        self.require_owner()?;
        let deadline = deadline.min(self.terminal_deadline);
        child_exited_without_reap(self.child.as_mut().expect("child guard is armed"), deadline)
    }

    #[cfg(not(unix))]
    pub(crate) fn exited_without_reap(
        &mut self,
        _deadline: Instant,
    ) -> NativeExecutionResult<bool> {
        self.require_owner()?;
        Ok(self
            .child
            .as_mut()
            .expect("child guard is armed")
            .try_wait()?
            .is_some())
    }

    #[cfg(unix)]
    fn group_members(
        &self,
        process_group_id: i32,
        excluded_leader: i32,
    ) -> NativeExecutionResult<Vec<i32>> {
        self.require_owner()?;
        #[cfg(test)]
        if self.cleanup_fault == Some(NativeExecutionCleanupFault::PersistentEnumeration) {
            return invalid("injected persistent process-group enumeration failure");
        }
        process_group_members(process_group_id, excluded_leader)
    }

    #[cfg(unix)]
    fn signal_group_errno(&self, process_group_id: i32) -> NativeExecutionResult<Option<i32>> {
        self.require_owner()?;
        #[cfg(test)]
        {
            self.signal_attempt_count
                .set(self.signal_attempt_count.get().saturating_add(1));
            if self.cleanup_fault == Some(NativeExecutionCleanupFault::PersistentSignalEperm) {
                return Ok(Some(libc::EPERM));
            }
        }
        // SAFETY: the direct child remains WNOWAIT-reserved and therefore keeps its
        // evaluator-owned process-group ID from being reused until stable descendant absence is
        // proven.
        signal_process_group_errno(process_group_id)
    }

    pub(crate) fn abandon_after_cleanup_failure(&mut self, label: &str) -> Option<String> {
        if self.require_owner().is_err() {
            self.child.take();
            return Some(format!(
                "{label} released a foreign-process guard copy without signaling its owner process group"
            ));
        }
        let child = self.child.take()?;
        let pid = child.id();
        drop(child);
        Some(format!(
            "{label} abandoned evaluator-owned PID {pid} after cleanup authority failed; collector/process resources may remain detached and this result is non-authorizing"
        ))
    }

    pub(crate) fn terminate_group_and_reap_with_status_until(
        &mut self,
        deadline: Instant,
    ) -> NativeExecutionCleanupOutcome {
        if let Err(error) = self.require_owner() {
            // A forked evaluator copy owns only duplicated local handles. Releasing them prevents
            // pipe interference while preserving the original evaluator's process-group authority.
            self.child.take();
            return NativeExecutionCleanupOutcome::CleanupUnproven(error);
        }
        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_cleanup_attempt) {
            return NativeExecutionCleanupOutcome::CleanupUnproven(NativeExecutionError::Invalid(
                "injected one-shot post-spawn cleanup uncertainty".into(),
            ));
        }
        let result = match self.group_authority {
            NativeExecutionGroupAuthority::VerifiedLeader => {
                self.terminate_verified_group_and_reap_with_status_until(deadline)
            }
            NativeExecutionGroupAuthority::ConstructedUnverified => {
                self.terminate_constructed_group_and_reap_with_status_until(deadline)
            }
            NativeExecutionGroupAuthority::UnownedUnverified => {
                match self.terminate_unowned_direct_child_until(deadline) {
                    Ok(_) => invalid(
                        "direct child was reaped but descendant absence cannot be proven without verified or construction-owned group authority",
                    ),
                    Err(error) => Err(error),
                }
            }
        };
        match result {
            Ok(status) => NativeExecutionCleanupOutcome::ProvenTerminal(status),
            Err(error) => NativeExecutionCleanupOutcome::CleanupUnproven(error),
        }
    }

    fn terminate_verified_group_and_reap_with_status_until(
        &mut self,
        deadline: Instant,
    ) -> NativeExecutionResult<ExitStatus> {
        let deadline = deadline.min(self.terminal_deadline);
        let Some(mut child) = self.child.take() else {
            return invalid("child guard was already disarmed");
        };
        #[cfg(unix)]
        {
            let Some(process_group_id) = self.process_group_id else {
                self.child = Some(child);
                return invalid("verified child guard omitted its process-group identity");
            };
            let child_pid = match i32::try_from(child.id()) {
                Ok(pid) => pid,
                Err(_) => {
                    self.child = Some(child);
                    return invalid("direct child PID exceeded the process-group boundary");
                }
            };
            if child_pid != process_group_id {
                self.child = Some(child);
                return invalid("direct-child identity drifted from its verified process group");
            }
            #[cfg(test)]
            let leader_observation = if self.force_initial_observation_deadline_interrupt {
                Ok(NativeExecutionLeaderObservation::InterruptedAtDeadline)
            } else {
                observe_child_exit_without_reap(&mut child, deadline)
            };
            #[cfg(not(test))]
            let leader_observation = observe_child_exit_without_reap(&mut child, deadline);
            let leader_observation = match leader_observation {
                Ok(observation) => observation,
                Err(error) => {
                    self.child = Some(child);
                    return Err(error);
                }
            };

            // Signal before any fallible descendant enumeration. A deadline-interrupted WNOWAIT
            // observation is retained as a terminal error, but it cannot suppress this first
            // best-effort signal while the verified leader remains unreaped. ECHILD and other
            // observation failures return above, before signaling an identity no longer reserved
            // by this guard.
            let errno = match self.signal_group_errno(process_group_id) {
                Ok(errno) => errno,
                Err(error) => {
                    self.child = Some(child);
                    return Err(error);
                }
            };
            let leader_exited = match leader_observation {
                NativeExecutionLeaderObservation::Known(exited) => exited,
                NativeExecutionLeaderObservation::InterruptedAtDeadline => {
                    let signal_detail = errno
                        .map(|errno| {
                            format!("; process-group signal also failed with errno {errno}")
                        })
                        .unwrap_or_default();
                    self.child = Some(child);
                    return invalid(format!(
                        "direct-child observation remained interrupted through its absolute deadline after the first process-group signal attempt{signal_detail}"
                    ));
                }
            };
            let signal_leader_exited = if errno == Some(libc::ESRCH) && !leader_exited {
                match child_exited_without_reap(&mut child, deadline) {
                    Ok(exited) => exited,
                    Err(error) => {
                        self.child = Some(child);
                        return Err(error);
                    }
                }
            } else {
                leader_exited
            };
            // macOS may report EPERM when the reserved group contains only its zombie leader.
            // Accept that one case only after a post-signal exact enumeration proves there are no
            // descendants. Enumeration still occurs after the best-effort signal, so an
            // enumerator failure cannot suppress the first termination attempt.
            let signal_validation = if errno == Some(libc::EPERM) && signal_leader_exited {
                match self.group_members(process_group_id, child_pid) {
                    Ok(members) if members.is_empty() => Ok(()),
                    Ok(_) => validate_group_signal_errno(errno, signal_leader_exited),
                    Err(error) => Err(error),
                }
            } else {
                validate_group_signal_errno(errno, signal_leader_exited)
            };
            if let Err(error) = signal_validation {
                self.child = Some(child);
                return Err(error);
            }
            if errno.is_none() {
                let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                    self.child = Some(child);
                    return invalid(
                        "child process group exhausted its cleanup deadline after termination",
                    );
                };
                thread::sleep(remaining.min(Duration::from_millis(10)));
            }
            if let Err(error) = require_stable_group_absence(
                deadline,
                || self.group_members(process_group_id, child_pid),
                || child_exited_without_reap(&mut child, deadline),
                || self.signal_group_errno(process_group_id),
            ) {
                self.child = Some(child);
                return Err(error);
            }

            loop {
                match child.try_wait() {
                    Ok(Some(status)) => return Ok(status),
                    Ok(None) if Instant::now() < deadline => {
                        let remaining = deadline.saturating_duration_since(Instant::now());
                        thread::sleep(remaining.min(Duration::from_millis(10)));
                    }
                    Ok(None) => {
                        self.child = Some(child);
                        return invalid(
                            "direct child was not reaped before the absolute cleanup deadline",
                        );
                    }
                    Err(error) => {
                        self.child = Some(child);
                        return Err(error.into());
                    }
                }
            }
        }

        #[cfg(not(unix))]
        {
            self.child = Some(child);
            invalid("owned process-group execution requires a supported Unix host")
        }
    }

    #[cfg(unix)]
    fn terminate_constructed_group_and_reap_with_status_until(
        &mut self,
        deadline: Instant,
    ) -> NativeExecutionResult<ExitStatus> {
        let deadline = deadline.min(self.terminal_deadline);
        let Some(mut child) = self.child.take() else {
            return invalid("child guard was already disarmed");
        };
        let Some(process_group_id) = self.process_group_id else {
            let _ = child.kill();
            self.child = Some(child);
            return invalid(
                "construction-owned child omitted a representable process-group identity",
            );
        };
        if i32::try_from(child.id()).ok() != Some(process_group_id) {
            self.child = Some(child);
            return invalid("construction-owned direct-child identity drifted");
        }

        // The command was configured with process_group(0), so this group belongs to the spawned
        // PID even when post-spawn verification failed because the child moved away from it. Keep
        // that PID unreaped, signal the constructed group first, and separately terminate the
        // direct child before proving both are terminal.
        let _ = self.signal_group_errno(process_group_id);
        let _ = child.kill();

        let mut consecutive_empty = 0_u8;
        loop {
            if Instant::now() >= deadline {
                self.child = Some(child);
                return invalid(
                    "constructed process group did not reach stable absence before the absolute terminal deadline",
                );
            }
            let members = match self.group_members(process_group_id, process_group_id) {
                Ok(members) => members,
                Err(error) => {
                    self.child = Some(child);
                    return Err(error);
                }
            };
            if Instant::now() > deadline {
                self.child = Some(child);
                return invalid(
                    "constructed process-group observation crossed the absolute terminal deadline",
                );
            }
            if members.is_empty() {
                consecutive_empty += 1;
                if consecutive_empty >= 2 {
                    break;
                }
            } else {
                consecutive_empty = 0;
                match self.signal_group_errno(process_group_id) {
                    Ok(None) | Ok(Some(libc::ESRCH)) => {}
                    Ok(Some(errno)) => {
                        self.child = Some(child);
                        return invalid(format!(
                            "constructed process-group signal failed with errno {errno}"
                        ));
                    }
                    Err(error) => {
                        self.child = Some(child);
                        return Err(error);
                    }
                }
            }
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                self.child = Some(child);
                return invalid(
                    "constructed process group did not reach stable absence before the absolute terminal deadline",
                );
            };
            thread::sleep(remaining.min(Duration::from_millis(10)));
        }

        loop {
            match child.try_wait() {
                Ok(Some(status)) => return Ok(status),
                Ok(None) if Instant::now() < deadline => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    thread::sleep(remaining.min(Duration::from_millis(10)));
                }
                Ok(None) => {
                    self.child = Some(child);
                    return invalid(
                        "construction-owned direct child was not reaped before the absolute cleanup deadline",
                    );
                }
                Err(error) => {
                    self.child = Some(child);
                    return Err(error.into());
                }
            }
        }
    }

    #[cfg(not(unix))]
    fn terminate_constructed_group_and_reap_with_status_until(
        &mut self,
        _deadline: Instant,
    ) -> NativeExecutionResult<ExitStatus> {
        invalid("owned process-group execution requires a supported Unix host")
    }

    fn terminate_unowned_direct_child_until(
        &mut self,
        deadline: Instant,
    ) -> NativeExecutionResult<ExitStatus> {
        let deadline = deadline.min(self.terminal_deadline);
        let Some(mut child) = self.child.take() else {
            return invalid("child guard was already disarmed");
        };
        let _ = child.kill();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return Ok(status),
                Ok(None) if Instant::now() < deadline => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    thread::sleep(remaining.min(Duration::from_millis(10)));
                }
                Ok(None) => {
                    self.child = Some(child);
                    return invalid(
                        "unowned direct child was not reaped before the absolute cleanup deadline",
                    );
                }
                Err(error) => {
                    self.child = Some(child);
                    return Err(error.into());
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn terminate_group_and_reap(&mut self) -> NativeExecutionResult<()> {
        if self.child.is_none() {
            return Ok(());
        }
        match self.terminate_group_and_reap_with_status_until(self.terminal_deadline) {
            NativeExecutionCleanupOutcome::ProvenTerminal(_) => Ok(()),
            NativeExecutionCleanupOutcome::CleanupUnproven(error) => Err(error),
        }
    }
}

#[cfg(unix)]
enum NativeExecutionLeaderObservation {
    Known(bool),
    InterruptedAtDeadline,
}

#[cfg(unix)]
fn observe_child_exit_without_reap(
    child: &mut Child,
    deadline: Instant,
) -> NativeExecutionResult<NativeExecutionLeaderObservation> {
    let process_id = child.id();
    loop {
        // SAFETY: waitid observes only the evaluator-owned direct child and WNOWAIT keeps the
        // group-leading PID reserved until group termination has been issued.
        let mut info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                process_id as libc::id_t,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if result == 0 {
            // SAFETY: waitid initialized the siginfo structure on success.
            return Ok(NativeExecutionLeaderObservation::Known(
                unsafe { info.si_pid() } != 0,
            ));
        }
        let error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::Interrupted {
            if Instant::now() >= deadline {
                return Ok(NativeExecutionLeaderObservation::InterruptedAtDeadline);
            }
            continue;
        }
        if error.raw_os_error() == Some(libc::ECHILD) {
            return invalid(
                "direct child was reaped outside its guard; process-group identity is no longer safe to signal",
            );
        }
        return Err(error.into());
    }
}

#[cfg(unix)]
fn child_exited_without_reap(child: &mut Child, deadline: Instant) -> NativeExecutionResult<bool> {
    match observe_child_exit_without_reap(child, deadline)? {
        NativeExecutionLeaderObservation::Known(exited) => Ok(exited),
        NativeExecutionLeaderObservation::InterruptedAtDeadline => {
            invalid("direct-child observation remained interrupted through its absolute deadline")
        }
    }
}

#[cfg(unix)]
fn signal_process_group_errno(process_group_id: i32) -> NativeExecutionResult<Option<i32>> {
    // SAFETY: callers retain an unreaped direct child whose PID is the exact process-group ID,
    // preventing reuse while the evaluator signals that group.
    if unsafe { libc::kill(-process_group_id, libc::SIGKILL) } == 0 {
        Ok(None)
    } else {
        std::io::Error::last_os_error()
            .raw_os_error()
            .map(Some)
            .ok_or_else(|| {
                NativeExecutionError::Invalid(
                    "process-group signal failed without a Unix errno".into(),
                )
            })
    }
}

#[cfg(unix)]
fn require_stable_group_absence(
    deadline: Instant,
    mut members: impl FnMut() -> NativeExecutionResult<Vec<i32>>,
    mut leader_exited: impl FnMut() -> NativeExecutionResult<bool>,
    mut signal: impl FnMut() -> NativeExecutionResult<Option<i32>>,
) -> NativeExecutionResult<()> {
    let mut consecutive_empty = 0_u8;
    loop {
        if Instant::now() >= deadline {
            return invalid(
                "child process group exhausted its absolute cleanup deadline before observation",
            );
        }
        let exited = leader_exited()?;
        let observed = members()?;
        if Instant::now() > deadline {
            return invalid(
                "child process group observation crossed the absolute cleanup deadline",
            );
        }
        if exited && observed.is_empty() {
            consecutive_empty += 1;
            if consecutive_empty >= 2 {
                return Ok(());
            }
        } else {
            consecutive_empty = 0;
            let errno = signal()?;
            let signal_leader_exited = if errno == Some(libc::ESRCH) && !exited {
                leader_exited()?
            } else {
                exited
            };
            validate_group_signal_errno(errno, signal_leader_exited)?;
        }
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            return invalid(
                "child process group did not reach stable absence before the absolute cleanup deadline",
            );
        };
        thread::sleep(remaining.min(Duration::from_millis(10)));
    }
}

#[cfg(unix)]
fn validate_group_signal_errno(
    errno: Option<i32>,
    leader_exited: bool,
) -> NativeExecutionResult<()> {
    match errno {
        None => Ok(()),
        Some(libc::ESRCH) if leader_exited => Ok(()),
        _ => invalid(format!(
            "process-group signal failed with errno {:?}",
            errno
        )),
    }
}

#[cfg(target_os = "macos")]
fn process_group_members(
    process_group_id: i32,
    excluded_leader: i32,
) -> NativeExecutionResult<Vec<i32>> {
    const PROC_ALL_PIDS: u32 = 1;
    const MAX_PROCESS_IDS: usize = 65_536;
    let mut pids = vec![0i32; MAX_PROCESS_IDS];
    let byte_capacity = i32::try_from(pids.len() * std::mem::size_of::<i32>()).map_err(|_| {
        NativeExecutionError::Invalid("process enumeration byte capacity overflowed".into())
    })?;
    // SAFETY: proc_listpids writes at most byte_capacity bytes to the allocated i32 buffer.
    let observed =
        unsafe { libc::proc_listpids(PROC_ALL_PIDS, 0, pids.as_mut_ptr().cast(), byte_capacity) };
    if observed < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let observed = usize::try_from(observed).map_err(|_| {
        NativeExecutionError::Invalid("process enumeration returned a negative size".into())
    })?;
    if observed >= byte_capacity as usize || observed % std::mem::size_of::<i32>() != 0 {
        return invalid("process enumeration exceeded or misaligned its fixed boundary");
    }
    pids.truncate(observed / std::mem::size_of::<i32>());
    let mut members = Vec::new();
    for pid in pids
        .into_iter()
        .filter(|pid| *pid > 1 && *pid != excluded_leader)
    {
        // SAFETY: getpgid is a read-only kernel query for a bounded positive PID.
        let observed_group = unsafe { libc::getpgid(pid) };
        if observed_group == process_group_id {
            members.push(pid);
        } else if observed_group < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error.into());
            }
        }
    }
    Ok(members)
}

#[cfg(target_os = "linux")]
fn process_group_members(
    process_group_id: i32,
    excluded_leader: i32,
) -> NativeExecutionResult<Vec<i32>> {
    let mut members = Vec::new();
    let mut entries = 0usize;
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|value| value.parse::<i32>().ok())
            .filter(|pid| *pid > 1 && *pid != excluded_leader)
        else {
            continue;
        };
        entries += 1;
        if entries > 65_536 {
            return invalid("process enumeration exceeded its fixed boundary");
        }
        // SAFETY: getpgid is a read-only kernel query for a bounded positive PID.
        let observed_group = unsafe { libc::getpgid(pid) };
        if observed_group == process_group_id {
            members.push(pid);
        } else if observed_group < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error.into());
            }
        }
    }
    Ok(members)
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
fn process_group_members(
    _process_group_id: i32,
    _excluded_leader: i32,
) -> NativeExecutionResult<Vec<i32>> {
    invalid("exact process-group enumeration is unsupported on this Unix host")
}

impl Drop for NativeExecutionChildGuard {
    fn drop(&mut self) {
        if self.child.is_none() {
            return;
        }
        if std::process::id() != self.owner_pid {
            // Never signal or wait from a forked evaluator copy. Dropping only the copied Child
            // closes pipe handles in this process and leaves the owner process's guard authoritative.
            self.child.take();
            return;
        }
        if let NativeExecutionCleanupOutcome::CleanupUnproven(error) =
            self.terminate_group_and_reap_with_status_until(self.terminal_deadline)
        {
            eprintln!("native execution child cleanup failed: {error}");
        }
    }
}

fn require_owner_pid(owner_pid: u32) -> NativeExecutionResult<()> {
    let observed_pid = std::process::id();
    if observed_pid != owner_pid {
        return Err(NativeExecutionError::OwnerProcessMismatch {
            owner_pid,
            observed_pid,
        });
    }
    Ok(())
}

fn pipe_owner_io_error(error: NativeExecutionError) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::PermissionDenied, error)
}

fn invalid<T>(message: impl Into<String>) -> NativeExecutionResult<T> {
    Err(NativeExecutionError::Invalid(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawned(outcome: NativeExecutionSpawnOutcome) -> NativeExecutionChildGuard {
        match outcome {
            NativeExecutionSpawnOutcome::Spawned(guard) => guard,
            other => panic!("expected a spawned guard, got {other:?}"),
        }
    }

    fn cleanup_result(outcome: NativeExecutionCleanupOutcome) -> NativeExecutionResult<ExitStatus> {
        match outcome {
            NativeExecutionCleanupOutcome::ProvenTerminal(status) => Ok(status),
            NativeExecutionCleanupOutcome::CleanupUnproven(error) => Err(error),
        }
    }

    #[cfg(unix)]
    #[test]
    fn stable_group_absence_resignals_late_members_and_never_retries_signal_errors() {
        use std::collections::VecDeque;

        let mut members = VecDeque::from([
            Vec::<i32>::new(),
            vec![4_242],
            Vec::<i32>::new(),
            Vec::<i32>::new(),
        ]);
        let mut signal_count = 0_u32;
        require_stable_group_absence(
            Instant::now() + Duration::from_secs(1),
            || {
                members.pop_front().ok_or_else(|| {
                    NativeExecutionError::Invalid("scripted member observations exhausted".into())
                })
            },
            || Ok(true),
            || {
                signal_count += 1;
                Ok(None)
            },
        )
        .unwrap();
        assert_eq!(signal_count, 1);
        assert!(members.is_empty());

        let mut signal_count = 0_u32;
        let error = require_stable_group_absence(
            Instant::now() + Duration::from_secs(1),
            || Ok(vec![4_242]),
            || Ok(false),
            || {
                signal_count += 1;
                if signal_count == 1 {
                    Ok(Some(libc::EPERM))
                } else {
                    Ok(None)
                }
            },
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("errno Some(1)"));
        assert_eq!(
            signal_count, 1,
            "one observation may cause only one signal attempt"
        );
    }

    #[cfg(unix)]
    #[test]
    fn production_spawn_creates_and_verifies_its_process_group() {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        let mut guard = spawned(NativeExecutionChildGuard::spawn(command, deadline));
        assert!(guard.is_armed());
        assert!(guard.try_wait_for_test().unwrap().is_none());
        let process_id = i32::try_from(guard.process_id().unwrap()).unwrap();
        // SAFETY: getpgid only observes the live evaluator-owned test child.
        assert_eq!(unsafe { libc::getpgid(process_id) }, process_id);
        guard.terminate_group_and_reap().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unwind_drop_uses_the_retained_terminal_deadline() {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", "sleep 30 & wait"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let guard = spawned(NativeExecutionChildGuard::spawn(command, deadline));
        let process_group_id = i32::try_from(guard.process_id().unwrap()).unwrap();
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = guard;
            panic!("injected unwind");
        }));
        assert!(unwound.is_err());
        // SAFETY: signal zero only observes the evaluator-owned test group after guard Drop.
        assert_eq!(unsafe { libc::kill(-process_group_id, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }

    #[cfg(unix)]
    #[test]
    fn expired_deadline_does_not_suppress_the_first_group_signal() {
        let deadline = Instant::now() + Duration::from_millis(30);
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        let mut guard = spawned(NativeExecutionChildGuard::spawn(command, deadline));
        let process_id = i32::try_from(guard.process_id().unwrap()).unwrap();
        while Instant::now() <= deadline {
            thread::sleep(Duration::from_millis(2));
        }
        guard.inject_initial_observation_deadline_interrupt();
        let error = cleanup_result(guard.terminate_group_and_reap_with_status_until(deadline))
            .unwrap_err()
            .to_string();
        assert!(error.contains("observation remained interrupted"));
        assert_eq!(guard.signal_attempt_count.get(), 1);
        guard.abandon_after_cleanup_failure("expired-deadline test cleanup");

        let mut status = 0;
        loop {
            // SAFETY: waitpid targets only the direct child created by this test.
            let waited = unsafe { libc::waitpid(process_id, &mut status, 0) };
            if waited == process_id {
                break;
            }
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::EINTR)
            );
        }
        assert!(libc::WIFSIGNALED(status));
        assert_eq!(libc::WTERMSIG(status), libc::SIGKILL);
    }

    #[cfg(unix)]
    #[test]
    fn production_verification_failure_cleanup_proves_descendant_absence() {
        use std::io::{BufRead, BufReader};
        use std::os::unix::process::CommandExt;
        use std::process::Stdio;

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", "sleep 30 & echo ready; wait"])
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command.spawn().unwrap();
        let process_group_id = i32::try_from(child.id()).unwrap();
        let mut ready = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut ready)
            .unwrap();
        assert_eq!(ready, "ready\n");

        let guard = NativeExecutionChildGuard {
            child: Some(child),
            process_group_id: Some(process_group_id),
            group_authority: NativeExecutionGroupAuthority::ConstructedUnverified,
            owner_pid: std::process::id(),
            terminal_deadline: deadline,
            cleanup_fault: None,
            signal_attempt_count: Cell::new(0),
            force_initial_observation_deadline_interrupt: false,
            fail_next_cleanup_attempt: false,
        };
        match guard.reject_after_spawn(NativeExecutionError::Invalid(
            "injected verification failure".into(),
        )) {
            NativeExecutionSpawnOutcome::PostSpawnTerminal {
                process_id,
                status,
                error,
            } => {
                assert_eq!(process_id, process_group_id as u32);
                assert!(!status.success());
                assert_eq!(
                    error.to_string(),
                    "invalid native execution data: injected verification failure"
                );
            }
            other => panic!("verification cleanup was not proven terminal: {other:?}"),
        }
        // SAFETY: signal zero only observes the test-owned process group after bounded cleanup.
        assert_eq!(unsafe { libc::kill(-process_group_id, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }

    #[cfg(unix)]
    #[test]
    fn exited_empty_group_accepts_only_the_guarded_eperm_case() {
        let deadline = Instant::now() + Duration::from_secs(2);
        let command = Command::new("/usr/bin/true");
        let mut guard = spawned(NativeExecutionChildGuard::spawn(command, deadline));
        loop {
            if guard.exited_without_reap(deadline).unwrap() {
                break;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(2));
        }
        guard.inject_cleanup_fault(NativeExecutionCleanupFault::PersistentSignalEperm);
        let status =
            cleanup_result(guard.terminate_group_and_reap_with_status_until(deadline)).unwrap();
        assert!(status.success());
        assert_eq!(guard.signal_attempt_count.get(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn leader_exited_path_signals_before_an_enumerator_failure() {
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", "sleep 30 & exit 0"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let mut guard = spawned(NativeExecutionChildGuard::spawn(command, deadline));
        loop {
            if guard.exited_without_reap(deadline).unwrap() {
                break;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
        guard.inject_cleanup_fault(NativeExecutionCleanupFault::PersistentEnumeration);
        let error = cleanup_result(guard.terminate_group_and_reap_with_status_until(deadline))
            .unwrap_err()
            .to_string();
        assert!(error.contains("enumeration failure"));
        assert_eq!(guard.signal_attempt_count.get(), 1);

        // Test-only recovery prevents leaving the deliberately faulted process as residue.
        guard.cleanup_fault = None;
        guard.terminate_group_and_reap().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn forked_guard_copy_rejects_pipe_access_and_drop_never_signals_owner_group() {
        use std::process::Stdio;

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut command = Command::new("/bin/sleep");
        command
            .arg("30")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut guard = spawned(NativeExecutionChildGuard::spawn(command, deadline));
        let process_group_id = i32::try_from(guard.process_id().unwrap()).unwrap();

        // SAFETY: the forked branch performs only owner checking, descriptor-closing Drop, and
        // `_exit`; it never runs test-harness teardown or process-group operations.
        let forked_pid = unsafe { libc::fork() };
        assert!(
            forked_pid >= 0,
            "fork failed: {}",
            std::io::Error::last_os_error()
        );
        if forked_pid == 0 {
            let rejected = matches!(
                guard.take_stdin(),
                Err(NativeExecutionError::OwnerProcessMismatch { .. })
            );
            drop(guard);
            unsafe { libc::_exit(i32::from(!rejected)) };
        }

        let mut fork_status = 0;
        loop {
            // SAFETY: waitpid targets only the evaluator child created immediately above.
            let waited = unsafe { libc::waitpid(forked_pid, &mut fork_status, 0) };
            if waited == forked_pid {
                break;
            }
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::EINTR)
            );
        }
        assert!(libc::WIFEXITED(fork_status));
        assert_eq!(libc::WEXITSTATUS(fork_status), 0);
        // SAFETY: signal zero observes only the still-owner-held evaluator process group.
        assert_eq!(unsafe { libc::kill(-process_group_id, 0) }, 0);
        guard.terminate_group_and_reap().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn fork_after_pipe_extraction_rejects_every_inherited_pipe_operation() {
        use std::process::Stdio;

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut command = Command::new("/bin/sleep");
        command
            .arg("30")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut guard = spawned(NativeExecutionChildGuard::spawn(command, deadline));
        let process_group_id = i32::try_from(guard.process_id().unwrap()).unwrap();
        let mut stdin = guard.take_stdin().unwrap().unwrap();
        let mut stdout = guard.take_stdout().unwrap().unwrap();
        let mut stderr = guard.take_stderr().unwrap().unwrap();

        // SAFETY: the forked branch performs only owner-checked pipe calls, descriptor-closing
        // drops, guard Drop, and `_exit`; it never runs test-harness teardown or group operations.
        let forked_pid = unsafe { libc::fork() };
        assert!(
            forked_pid >= 0,
            "fork failed: {}",
            std::io::Error::last_os_error()
        );
        if forked_pid == 0 {
            let mut byte = [0_u8; 1];
            let rejected = stdin.write_all(b"foreign\n").is_err()
                && stdout.read(&mut byte).is_err()
                && stderr.read(&mut byte).is_err()
                && stdin.configure_nonblocking().is_err()
                && stdin.poll_writable(0).is_err();
            drop(stdin);
            drop(stdout);
            drop(stderr);
            drop(guard);
            unsafe { libc::_exit(i32::from(!rejected)) };
        }

        let mut fork_status = 0;
        loop {
            // SAFETY: waitpid targets only the evaluator child created immediately above.
            let waited = unsafe { libc::waitpid(forked_pid, &mut fork_status, 0) };
            if waited == forked_pid {
                break;
            }
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::EINTR)
            );
        }
        assert!(libc::WIFEXITED(fork_status));
        assert_eq!(libc::WEXITSTATUS(fork_status), 0);
        // SAFETY: signal zero observes only the still-owner-held evaluator process group.
        assert_eq!(unsafe { libc::kill(-process_group_id, 0) }, 0);
        stdin.configure_nonblocking().unwrap();
        assert!(stdin.poll_writable(0).unwrap().is_some());
        drop(stdin);
        drop(stdout);
        drop(stderr);
        guard.terminate_group_and_reap().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn post_spawn_cleanup_uncertainty_retains_an_owner_bound_quarantine() {
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "sleep 30 & wait"]);
        let mut guard = spawned(NativeExecutionChildGuard::spawn(command, deadline));
        let process_id = guard.process_id().unwrap();
        guard.inject_cleanup_fault(NativeExecutionCleanupFault::PersistentSignalEperm);

        let outcome = guard.reject_after_spawn(NativeExecutionError::Invalid(
            "injected post-spawn rejection".into(),
        ));
        let mut quarantine = match outcome {
            NativeExecutionSpawnOutcome::PostSpawnCleanupUnproven {
                process_id: observed,
                error,
                quarantine,
            } => {
                assert_eq!(observed, process_id);
                assert!(error.to_string().contains("cleanup remained unproven"));
                quarantine
            }
            other => panic!("cleanup uncertainty was erased: {other:?}"),
        };
        assert!(quarantine.is_armed());
        assert_eq!(quarantine.process_id().unwrap(), process_id);

        quarantine.cleanup_fault = None;
        quarantine.terminate_group_and_reap().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn one_shot_post_spawn_uncertainty_is_retried_by_owner_drop() {
        let deadline = Instant::now() + Duration::from_secs(3);
        inject_next_post_spawn_cleanup_unproven_for_test();
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "sleep 30 & wait"]);
        let (process_group_id, quarantine) =
            match NativeExecutionChildGuard::spawn(command, deadline) {
                NativeExecutionSpawnOutcome::PostSpawnCleanupUnproven {
                    process_id,
                    error,
                    quarantine,
                } => {
                    assert!(error.to_string().contains("cleanup remained unproven"));
                    (i32::try_from(process_id).unwrap(), quarantine)
                }
                other => panic!("one-shot cleanup uncertainty was not retained: {other:?}"),
            };
        // SAFETY: signal zero observes only the still-quarantined test process group.
        assert_eq!(unsafe { libc::kill(-process_group_id, 0) }, 0);
        drop(quarantine);
        // SAFETY: signal zero observes only the test group after the owner-bound Drop retry.
        assert_eq!(unsafe { libc::kill(-process_group_id, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_adoption_rejects_a_child_that_is_not_a_group_leader() {
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        let child = command.spawn().unwrap();
        let error = NativeExecutionChildGuard::adopt_spawned_group_for_test(
            child,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("not the leader of its own process group"));
    }

    #[cfg(unix)]
    #[test]
    fn signal_errno_validation_remains_closed_world() {
        assert!(validate_group_signal_errno(Some(libc::EPERM), true).is_err());
        assert!(validate_group_signal_errno(Some(libc::EPERM), false).is_err());
        assert!(validate_group_signal_errno(Some(libc::ESRCH), false).is_err());
        validate_group_signal_errno(Some(libc::ESRCH), true).unwrap();
    }
}
