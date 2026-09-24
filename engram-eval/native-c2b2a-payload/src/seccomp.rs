//! Deterministic, target-neutral model of the C2B2a seccomp contract.
//!
//! The builder emits classic BPF for the frozen AArch64 ABI.  The interpreter is deliberately
//! small and exists so that the complete decision surface can be tested on the macOS host without
//! installing a filter or starting a guest.  It never dereferences syscall pointer arguments.

use std::collections::BTreeSet;
use std::fmt;

/// Constants from the pinned Linux AArch64 syscall ABI.
pub mod aarch64 {
    pub const AUDIT_ARCH: u32 = 0xc000_00b7;

    pub const NR_SETXATTR: u32 = 5;
    pub const NR_LSETXATTR: u32 = 6;
    pub const NR_FSETXATTR: u32 = 7;
    pub const NR_GETXATTR: u32 = 8;
    pub const NR_LGETXATTR: u32 = 9;
    pub const NR_FGETXATTR: u32 = 10;
    pub const NR_REMOVEXATTR: u32 = 14;
    pub const NR_LREMOVEXATTR: u32 = 15;
    pub const NR_FREMOVEXATTR: u32 = 16;
    pub const NR_EVENTFD2: u32 = 19;
    pub const NR_EPOLL_CREATE1: u32 = 20;
    pub const NR_EPOLL_CTL: u32 = 21;
    pub const NR_EPOLL_PWAIT: u32 = 22;
    pub const NR_DUP: u32 = 23;
    pub const NR_DUP3: u32 = 24;
    pub const NR_FCNTL: u32 = 25;
    pub const NR_IOCTL: u32 = 29;
    pub const NR_MKDIRAT: u32 = 34;
    pub const NR_UNLINKAT: u32 = 35;
    pub const NR_LINKAT: u32 = 37;
    pub const NR_RENAMEAT: u32 = 38;
    pub const NR_UMOUNT2: u32 = 39;
    pub const NR_MOUNT: u32 = 40;
    pub const NR_PIVOT_ROOT: u32 = 41;
    pub const NR_FSTATFS: u32 = 44;
    pub const NR_FTRUNCATE: u32 = 46;
    pub const NR_FALLOCATE: u32 = 47;
    pub const NR_FACCESSAT: u32 = 48;
    pub const NR_CHROOT: u32 = 51;
    pub const NR_OPENAT: u32 = 56;
    pub const NR_CLOSE: u32 = 57;
    pub const NR_GETDENTS64: u32 = 61;
    pub const NR_LSEEK: u32 = 62;
    pub const NR_READ: u32 = 63;
    pub const NR_WRITE: u32 = 64;
    pub const NR_READV: u32 = 65;
    pub const NR_WRITEV: u32 = 66;
    pub const NR_PREAD64: u32 = 67;
    pub const NR_PWRITE64: u32 = 68;
    pub const NR_PSELECT6: u32 = 72;
    pub const NR_PPOLL: u32 = 73;
    pub const NR_READLINKAT: u32 = 78;
    pub const NR_NEWFSTATAT: u32 = 79;
    pub const NR_FSTAT: u32 = 80;
    pub const NR_FSYNC: u32 = 82;
    pub const NR_FDATASYNC: u32 = 83;
    pub const NR_SYNC_FILE_RANGE: u32 = 84;
    pub const NR_TIMERFD_CREATE: u32 = 85;
    pub const NR_TIMERFD_SETTIME: u32 = 86;
    pub const NR_TIMERFD_GETTIME: u32 = 87;
    pub const NR_EXIT: u32 = 93;
    pub const NR_EXIT_GROUP: u32 = 94;
    pub const NR_SET_TID_ADDRESS: u32 = 96;
    pub const NR_UNSHARE: u32 = 97;
    pub const NR_FUTEX: u32 = 98;
    pub const NR_SET_ROBUST_LIST: u32 = 99;
    pub const NR_NANOSLEEP: u32 = 101;
    pub const NR_CLOCK_GETTIME: u32 = 113;
    pub const NR_CLOCK_NANOSLEEP: u32 = 115;
    pub const NR_SCHED_SETAFFINITY: u32 = 122;
    pub const NR_SCHED_GETAFFINITY: u32 = 123;
    pub const NR_SCHED_YIELD: u32 = 124;
    pub const NR_RESTART_SYSCALL: u32 = 128;
    pub const NR_TGKILL: u32 = 131;
    pub const NR_SIGALTSTACK: u32 = 132;
    pub const NR_RT_SIGACTION: u32 = 134;
    pub const NR_RT_SIGPROCMASK: u32 = 135;
    pub const NR_RT_SIGRETURN: u32 = 139;
    pub const NR_TIMES: u32 = 153;
    pub const NR_UNAME: u32 = 160;
    pub const NR_GETRUSAGE: u32 = 165;
    pub const NR_UMASK: u32 = 166;
    pub const NR_PRCTL: u32 = 167;
    pub const NR_GETTIMEOFDAY: u32 = 169;
    pub const NR_GETPID: u32 = 172;
    pub const NR_GETPPID: u32 = 173;
    pub const NR_GETUID: u32 = 174;
    pub const NR_GETEUID: u32 = 175;
    pub const NR_GETGID: u32 = 176;
    pub const NR_GETEGID: u32 = 177;
    pub const NR_GETTID: u32 = 178;
    pub const NR_SYSINFO: u32 = 179;
    pub const NR_SOCKET: u32 = 198;
    pub const NR_SOCKETPAIR: u32 = 199;
    pub const NR_BIND: u32 = 200;
    pub const NR_LISTEN: u32 = 201;
    pub const NR_ACCEPT: u32 = 202;
    pub const NR_CONNECT: u32 = 203;
    pub const NR_GETSOCKNAME: u32 = 204;
    pub const NR_GETPEERNAME: u32 = 205;
    pub const NR_SENDTO: u32 = 206;
    pub const NR_RECVFROM: u32 = 207;
    pub const NR_SETSOCKOPT: u32 = 208;
    pub const NR_GETSOCKOPT: u32 = 209;
    pub const NR_SHUTDOWN: u32 = 210;
    pub const NR_SENDMSG: u32 = 211;
    pub const NR_RECVMSG: u32 = 212;
    pub const NR_READAHEAD: u32 = 213;
    pub const NR_BRK: u32 = 214;
    pub const NR_MUNMAP: u32 = 215;
    pub const NR_MREMAP: u32 = 216;
    pub const NR_CLONE: u32 = 220;
    pub const NR_EXECVE: u32 = 221;
    pub const NR_MMAP: u32 = 222;
    pub const NR_FADVISE64: u32 = 223;
    pub const NR_MPROTECT: u32 = 226;
    pub const NR_MINCORE: u32 = 232;
    pub const NR_MADVISE: u32 = 233;
    pub const NR_ACCEPT4: u32 = 242;
    pub const NR_RECVMMSG: u32 = 243;
    pub const NR_PRLIMIT64: u32 = 261;
    pub const NR_SENDMMSG: u32 = 269;
    pub const NR_RENAMEAT2: u32 = 276;
    pub const NR_SECCOMP: u32 = 277;
    pub const NR_GETRANDOM: u32 = 278;
    pub const NR_BPF: u32 = 280;
    pub const NR_EXECVEAT: u32 = 281;
    pub const NR_MEMBARRIER: u32 = 283;
    pub const NR_STATX: u32 = 291;
    pub const NR_RSEQ: u32 = 293;
    pub const NR_CLONE3: u32 = 435;
    pub const NR_CLOSE_RANGE: u32 = 436;
    pub const NR_FACCESSAT2: u32 = 439;
    pub const NR_EPOLL_PWAIT2: u32 = 441;
    pub const NR_FUTEX_WAITV: u32 = 449;
}

pub const EPERM: u16 = 1;
pub const ENOSYS: u16 = 38;
pub const EXPECTED_EXECVEAT_FD: u64 = 7;
pub const AT_EMPTY_PATH: u64 = 0x1000;
pub const PTHREAD_CLONE_FLAGS: u64 = 0x007d_0f00;

pub const PR_GET_PDEATHSIG: u64 = 2;
pub const PR_GET_DUMPABLE: u64 = 3;
pub const PR_SET_DUMPABLE: u64 = 4;
pub const PR_SET_NAME: u64 = 15;
pub const PR_GET_NAME: u64 = 16;
pub const SECCOMP_SET_MODE_FILTER: u64 = 1;

const SECCOMP_RET_KILL_PROCESS: u32 = 0x8000_0000;
const SECCOMP_RET_TRAP: u32 = 0x0003_0000;
const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
const SECCOMP_RET_ACTION_FULL: u32 = 0xffff_0000;

const BPF_LD_W_ABS: u16 = 0x20;
const BPF_JMP_JEQ_K: u16 = 0x15;
const BPF_RET_K: u16 = 0x06;

const OFFSET_NR: u32 = 0;
const OFFSET_ARCH: u32 = 4;
const OFFSET_INSTRUCTION_POINTER: u32 = 8;
const OFFSET_ARGS: u32 = 16;
const MAX_CLASSIC_BPF_INSTRUCTIONS: usize = 4096;

/// The frozen semantic `access` operation has no AArch64 syscall number.  Musl implements it
/// through `faccessat`; likewise `fork` and `vfork` use `clone` on this ABI.  The filter must not
/// invent syscall numbers for those libc wrappers.
pub const ABI_WRAPPERS_WITHOUT_SYSCALL_NUMBERS: &[&str] = &["access", "fork", "vfork"];

/// Syscalls explicitly allowed by the frozen steady-state contract.
pub const STEADY_ALLOWED_SYSCALLS: &[u32] = &[
    aarch64::NR_READ,
    aarch64::NR_WRITE,
    aarch64::NR_READV,
    aarch64::NR_WRITEV,
    aarch64::NR_PREAD64,
    aarch64::NR_PWRITE64,
    aarch64::NR_CLOSE,
    aarch64::NR_CLOSE_RANGE,
    aarch64::NR_DUP,
    aarch64::NR_DUP3,
    aarch64::NR_FCNTL,
    aarch64::NR_IOCTL,
    aarch64::NR_LSEEK,
    aarch64::NR_OPENAT,
    aarch64::NR_NEWFSTATAT,
    aarch64::NR_FSTAT,
    aarch64::NR_STATX,
    aarch64::NR_FSTATFS,
    aarch64::NR_GETDENTS64,
    aarch64::NR_READLINKAT,
    aarch64::NR_FACCESSAT,
    aarch64::NR_FACCESSAT2,
    aarch64::NR_MKDIRAT,
    aarch64::NR_UNLINKAT,
    aarch64::NR_RENAMEAT,
    aarch64::NR_RENAMEAT2,
    aarch64::NR_LINKAT,
    aarch64::NR_FTRUNCATE,
    aarch64::NR_FALLOCATE,
    aarch64::NR_FSYNC,
    aarch64::NR_FDATASYNC,
    aarch64::NR_SYNC_FILE_RANGE,
    aarch64::NR_READAHEAD,
    aarch64::NR_FADVISE64,
    aarch64::NR_MMAP,
    aarch64::NR_MPROTECT,
    aarch64::NR_MUNMAP,
    aarch64::NR_MREMAP,
    aarch64::NR_MADVISE,
    aarch64::NR_MINCORE,
    aarch64::NR_BRK,
    aarch64::NR_GETRANDOM,
    aarch64::NR_RT_SIGACTION,
    aarch64::NR_RT_SIGPROCMASK,
    aarch64::NR_RT_SIGRETURN,
    aarch64::NR_SIGALTSTACK,
    aarch64::NR_FUTEX,
    aarch64::NR_FUTEX_WAITV,
    aarch64::NR_SET_TID_ADDRESS,
    aarch64::NR_SET_ROBUST_LIST,
    aarch64::NR_RSEQ,
    aarch64::NR_MEMBARRIER,
    aarch64::NR_SCHED_YIELD,
    aarch64::NR_SCHED_GETAFFINITY,
    aarch64::NR_SCHED_SETAFFINITY,
    aarch64::NR_CLOCK_GETTIME,
    aarch64::NR_CLOCK_NANOSLEEP,
    aarch64::NR_NANOSLEEP,
    aarch64::NR_GETTIMEOFDAY,
    aarch64::NR_GETRUSAGE,
    aarch64::NR_TIMES,
    aarch64::NR_UNAME,
    aarch64::NR_SYSINFO,
    aarch64::NR_GETPID,
    aarch64::NR_GETPPID,
    aarch64::NR_GETTID,
    aarch64::NR_GETUID,
    aarch64::NR_GETEUID,
    aarch64::NR_GETGID,
    aarch64::NR_GETEGID,
    aarch64::NR_PRLIMIT64,
    aarch64::NR_UMASK,
    aarch64::NR_GETXATTR,
    aarch64::NR_LGETXATTR,
    aarch64::NR_FGETXATTR,
    aarch64::NR_SETXATTR,
    aarch64::NR_LSETXATTR,
    aarch64::NR_FSETXATTR,
    aarch64::NR_REMOVEXATTR,
    aarch64::NR_LREMOVEXATTR,
    aarch64::NR_FREMOVEXATTR,
    aarch64::NR_EPOLL_CREATE1,
    aarch64::NR_EPOLL_CTL,
    aarch64::NR_EPOLL_PWAIT,
    aarch64::NR_EPOLL_PWAIT2,
    aarch64::NR_EVENTFD2,
    aarch64::NR_TIMERFD_CREATE,
    aarch64::NR_TIMERFD_SETTIME,
    aarch64::NR_TIMERFD_GETTIME,
    aarch64::NR_PPOLL,
    aarch64::NR_PSELECT6,
    aarch64::NR_RESTART_SYSCALL,
    aarch64::NR_TGKILL,
    aarch64::NR_EXIT,
    aarch64::NR_EXIT_GROUP,
];

/// Every application-network syscall receives `EPERM` in both filters.
pub const NETWORK_SYSCALLS: &[u32] = &[
    aarch64::NR_SOCKET,
    aarch64::NR_SOCKETPAIR,
    aarch64::NR_CONNECT,
    aarch64::NR_BIND,
    aarch64::NR_LISTEN,
    aarch64::NR_ACCEPT,
    aarch64::NR_ACCEPT4,
    aarch64::NR_SENDTO,
    aarch64::NR_SENDMSG,
    aarch64::NR_SENDMMSG,
    aarch64::NR_RECVFROM,
    aarch64::NR_RECVMSG,
    aarch64::NR_RECVMMSG,
    aarch64::NR_GETSOCKNAME,
    aarch64::NR_GETPEERNAME,
    aarch64::NR_SETSOCKOPT,
    aarch64::NR_GETSOCKOPT,
    aarch64::NR_SHUTDOWN,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    KillProcess,
    Trap,
    Errno(u16),
    Allow,
}

impl Action {
    pub const fn raw(self) -> u32 {
        match self {
            Self::KillProcess => SECCOMP_RET_KILL_PROCESS,
            Self::Trap => SECCOMP_RET_TRAP,
            Self::Errno(errno) => SECCOMP_RET_ERRNO | errno as u32,
            Self::Allow => SECCOMP_RET_ALLOW,
        }
    }

    fn from_raw(raw: u32) -> Result<Self, InterpretError> {
        match raw & SECCOMP_RET_ACTION_FULL {
            SECCOMP_RET_KILL_PROCESS => Ok(Self::KillProcess),
            SECCOMP_RET_TRAP => Ok(Self::Trap),
            SECCOMP_RET_ERRNO => Ok(Self::Errno((raw & 0xffff) as u16)),
            SECCOMP_RET_ALLOW => Ok(Self::Allow),
            _ => Err(InterpretError::UnknownReturn(raw)),
        }
    }

    const fn precedence(self) -> u8 {
        match self {
            Self::KillProcess => 3,
            Self::Trap => 2,
            Self::Errno(_) => 1,
            Self::Allow => 0,
        }
    }
}

/// Layout-compatible with Linux `struct sock_filter`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Instruction {
    pub code: u16,
    pub jt: u8,
    pub jf: u8,
    pub k: u32,
}

impl Instruction {
    const fn statement(code: u16, k: u32) -> Self {
        Self {
            code,
            jt: 0,
            jf: 0,
            k,
        }
    }

    const fn jump_eq(k: u32, jt: u8, jf: u8) -> Self {
        Self {
            code: BPF_JMP_JEQ_K,
            jt,
            jf,
            k,
        }
    }

    const fn return_action(action: Action) -> Self {
        Self::statement(BPF_RET_K, action.raw())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Stable little-endian encoding used only for build records and golden tests.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.instructions.len() * 8);
        for instruction in &self.instructions {
            bytes.extend_from_slice(&instruction.code.to_le_bytes());
            bytes.push(instruction.jt);
            bytes.push(instruction.jf);
            bytes.extend_from_slice(&instruction.k.to_le_bytes());
        }
        bytes
    }

    pub fn validate(&self) -> Result<(), ProgramError> {
        if self.instructions.is_empty() {
            return Err(ProgramError::Empty);
        }
        if self.instructions.len() > MAX_CLASSIC_BPF_INSTRUCTIONS {
            return Err(ProgramError::TooLong(self.instructions.len()));
        }

        for (index, instruction) in self.instructions.iter().enumerate() {
            match instruction.code {
                BPF_LD_W_ABS => {
                    if load_offset_kind(instruction.k).is_none() {
                        return Err(ProgramError::InvalidLoadOffset {
                            index,
                            offset: instruction.k,
                        });
                    }
                    if index + 1 >= self.instructions.len() {
                        return Err(ProgramError::FallsOffEnd(index));
                    }
                }
                BPF_JMP_JEQ_K => {
                    for distance in [instruction.jt, instruction.jf] {
                        let target = index + 1 + usize::from(distance);
                        if target >= self.instructions.len() {
                            return Err(ProgramError::InvalidJump { index, target });
                        }
                    }
                }
                BPF_RET_K => {
                    Action::from_raw(instruction.k)
                        .map_err(|_| ProgramError::InvalidReturn(index, instruction.k))?;
                }
                code => return Err(ProgramError::UnsupportedInstruction(index, code)),
            }
        }

        if self.instructions.last().map(|instruction| instruction.code) != Some(BPF_RET_K) {
            return Err(ProgramError::FallsOffEnd(self.instructions.len() - 1));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuildError {
    DuplicateSyscall(u32),
    RuleTooLong(usize),
    InvalidProgram(ProgramError),
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSyscall(nr) => write!(formatter, "duplicate syscall rule {nr}"),
            Self::RuleTooLong(len) => write!(formatter, "classic-BPF rule has {len} instructions"),
            Self::InvalidProgram(error) => write!(formatter, "invalid seccomp program: {error}"),
        }
    }
}

impl std::error::Error for BuildError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramError {
    Empty,
    TooLong(usize),
    InvalidLoadOffset { index: usize, offset: u32 },
    InvalidJump { index: usize, target: usize },
    InvalidReturn(usize, u32),
    UnsupportedInstruction(usize, u16),
    FallsOffEnd(usize),
}

impl fmt::Display for ProgramError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("empty classic-BPF program"),
            Self::TooLong(len) => write!(formatter, "{len} instructions exceeds the kernel limit"),
            Self::InvalidLoadOffset { index, offset } => {
                write!(
                    formatter,
                    "instruction {index} loads invalid offset {offset}"
                )
            }
            Self::InvalidJump { index, target } => {
                write!(formatter, "instruction {index} jumps to {target}")
            }
            Self::InvalidReturn(index, raw) => {
                write!(
                    formatter,
                    "instruction {index} returns unknown action {raw:#x}"
                )
            }
            Self::UnsupportedInstruction(index, code) => {
                write!(
                    formatter,
                    "instruction {index} has unsupported opcode {code:#x}"
                )
            }
            Self::FallsOffEnd(index) => write!(formatter, "instruction {index} falls off the end"),
        }
    }
}

impl std::error::Error for ProgramError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InterpretError {
    InvalidProgram(ProgramError),
    UnknownReturn(u32),
    NoReturn,
}

impl fmt::Display for InterpretError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProgram(error) => write!(formatter, "invalid seccomp program: {error}"),
            Self::UnknownReturn(raw) => write!(formatter, "unknown seccomp return {raw:#x}"),
            Self::NoReturn => formatter.write_str("classic-BPF program did not return"),
        }
    }
}

impl std::error::Error for InterpretError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SeccompInput {
    pub nr: i32,
    pub arch: u32,
    pub instruction_pointer: u64,
    pub args: [u64; 6],
}

impl SeccompInput {
    pub const fn aarch64(nr: u32, args: [u64; 6]) -> Self {
        Self {
            nr: nr as i32,
            arch: aarch64::AUDIT_ARCH,
            instruction_pointer: 0,
            args,
        }
    }
}

#[derive(Clone, Copy)]
enum LoadOffset {
    Nr,
    Arch,
    InstructionPointerLow,
    InstructionPointerHigh,
    ArgumentLow(usize),
    ArgumentHigh(usize),
}

fn load_offset_kind(offset: u32) -> Option<LoadOffset> {
    match offset {
        OFFSET_NR => Some(LoadOffset::Nr),
        OFFSET_ARCH => Some(LoadOffset::Arch),
        OFFSET_INSTRUCTION_POINTER => Some(LoadOffset::InstructionPointerLow),
        value if value == OFFSET_INSTRUCTION_POINTER + 4 => {
            Some(LoadOffset::InstructionPointerHigh)
        }
        value if (OFFSET_ARGS..OFFSET_ARGS + 48).contains(&value) => {
            let relative = value - OFFSET_ARGS;
            let argument = (relative / 8) as usize;
            match relative % 8 {
                0 => Some(LoadOffset::ArgumentLow(argument)),
                4 => Some(LoadOffset::ArgumentHigh(argument)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn load_word(input: &SeccompInput, offset: u32) -> Option<u32> {
    match load_offset_kind(offset)? {
        LoadOffset::Nr => Some(input.nr as u32),
        LoadOffset::Arch => Some(input.arch),
        LoadOffset::InstructionPointerLow => Some(input.instruction_pointer as u32),
        LoadOffset::InstructionPointerHigh => Some((input.instruction_pointer >> 32) as u32),
        LoadOffset::ArgumentLow(index) => Some(input.args[index] as u32),
        LoadOffset::ArgumentHigh(index) => Some((input.args[index] >> 32) as u32),
    }
}

struct Builder {
    instructions: Vec<Instruction>,
    syscall_rules: BTreeSet<u32>,
}

impl Builder {
    fn new() -> Self {
        Self {
            instructions: vec![
                Instruction::statement(BPF_LD_W_ABS, OFFSET_ARCH),
                Instruction::jump_eq(aarch64::AUDIT_ARCH, 1, 0),
                Instruction::return_action(Action::KillProcess),
                Instruction::statement(BPF_LD_W_ABS, OFFSET_NR),
            ],
            syscall_rules: BTreeSet::new(),
        }
    }

    fn syscall(&mut self, nr: u32, block: Vec<Instruction>) -> Result<(), BuildError> {
        if !self.syscall_rules.insert(nr) {
            return Err(BuildError::DuplicateSyscall(nr));
        }
        let skip = u8::try_from(block.len()).map_err(|_| BuildError::RuleTooLong(block.len()))?;
        self.instructions.push(Instruction::jump_eq(nr, 0, skip));
        self.instructions.extend(block);
        Ok(())
    }

    fn simple(&mut self, nr: u32, action: Action) -> Result<(), BuildError> {
        self.syscall(nr, vec![Instruction::return_action(action)])
    }

    fn finish(mut self) -> Result<Program, BuildError> {
        self.instructions
            .push(Instruction::return_action(Action::Trap));
        let program = Program {
            instructions: self.instructions,
        };
        program.validate().map_err(BuildError::InvalidProgram)?;
        Ok(program)
    }
}

fn argument_offset(index: usize) -> u32 {
    OFFSET_ARGS + (index as u32 * 8)
}

fn require_word(block: &mut Vec<Instruction>, offset: u32, expected: u32, failure: Action) {
    block.push(Instruction::statement(BPF_LD_W_ABS, offset));
    block.push(Instruction::jump_eq(expected, 1, 0));
    block.push(Instruction::return_action(failure));
}

fn require_u64(block: &mut Vec<Instruction>, argument: usize, expected: u64, failure: Action) {
    require_word(block, argument_offset(argument), expected as u32, failure);
    require_word(
        block,
        argument_offset(argument) + 4,
        (expected >> 32) as u32,
        failure,
    );
}

fn exact_execveat_block() -> Vec<Instruction> {
    let mut block = Vec::new();
    require_u64(&mut block, 0, EXPECTED_EXECVEAT_FD, Action::Errno(EPERM));
    // Classic BPF cannot dereference args[1].  The zero-length pathname byte is bound by the
    // accepted call site and source/ELF firewall, while this filter binds FD and flags.
    require_u64(&mut block, 4, AT_EMPTY_PATH, Action::Errno(EPERM));
    block.push(Instruction::return_action(Action::Allow));
    block
}

fn clone_block() -> Vec<Instruction> {
    let mut block = Vec::new();
    require_u64(&mut block, 0, PTHREAD_CLONE_FLAGS, Action::Errno(EPERM));
    block.push(Instruction::return_action(Action::Allow));
    block
}

fn prctl_block(bootstrap: bool) -> Vec<Instruction> {
    let mut block = Vec::new();
    require_word(&mut block, argument_offset(0) + 4, 0, Action::Trap);
    block.push(Instruction::statement(BPF_LD_W_ABS, argument_offset(0)));

    let options: &[u64] = if bootstrap {
        &[PR_GET_PDEATHSIG, PR_GET_DUMPABLE, PR_SET_NAME, PR_GET_NAME]
    } else {
        &[PR_SET_NAME, PR_GET_NAME]
    };
    for option in options {
        block.push(Instruction::jump_eq(*option as u32, 0, 1));
        block.push(Instruction::return_action(Action::Allow));
    }

    if bootstrap {
        // If the option is not PR_SET_DUMPABLE, skip its seven-instruction argument check and
        // reach the terminal trap.  PR_SET_DUMPABLE is permitted only with value zero.
        block.push(Instruction::jump_eq(PR_SET_DUMPABLE as u32, 0, 7));
        require_u64(&mut block, 1, 0, Action::Trap);
        block.push(Instruction::return_action(Action::Allow));
    }

    block.push(Instruction::return_action(Action::Trap));
    block
}

fn seccomp_installer_block() -> Vec<Instruction> {
    let mut block = Vec::new();
    require_u64(&mut block, 0, SECCOMP_SET_MODE_FILTER, Action::Trap);
    require_u64(&mut block, 1, 0, Action::Trap);
    block.push(Instruction::return_action(Action::Allow));
    block
}

fn add_common_rules(builder: &mut Builder, clone3_action: Action) -> Result<(), BuildError> {
    for &nr in NETWORK_SYSCALLS {
        builder.simple(nr, Action::Errno(EPERM))?;
    }
    builder.simple(aarch64::NR_EXECVE, Action::Errno(EPERM))?;
    builder.simple(aarch64::NR_CLONE3, clone3_action)?;
    builder.syscall(aarch64::NR_CLONE, clone_block())?;
    Ok(())
}

fn add_steady_allowlist(builder: &mut Builder) -> Result<(), BuildError> {
    for &nr in STEADY_ALLOWED_SYSCALLS {
        builder.simple(nr, Action::Allow)?;
    }
    Ok(())
}

/// Builds the pre-exec filter.  It is a superset of steady-state ALLOW decisions because filters
/// remain stacked after `execveat`; the newer steady filter supplies the tighter final decision.
pub fn bootstrap_program() -> Result<Program, BuildError> {
    let mut builder = Builder::new();
    add_common_rules(&mut builder, Action::Errno(EPERM))?;
    builder.syscall(aarch64::NR_EXECVEAT, exact_execveat_block())?;
    builder.syscall(aarch64::NR_PRCTL, prctl_block(true))?;
    builder.syscall(aarch64::NR_SECCOMP, seccomp_installer_block())?;
    add_steady_allowlist(&mut builder)?;
    builder.finish()
}

/// Builds the closed steady-state collector filter.
pub fn steady_program() -> Result<Program, BuildError> {
    let mut builder = Builder::new();
    add_common_rules(&mut builder, Action::Errno(ENOSYS))?;
    builder.simple(aarch64::NR_EXECVEAT, Action::Errno(EPERM))?;
    builder.syscall(aarch64::NR_PRCTL, prctl_block(false))?;
    add_steady_allowlist(&mut builder)?;
    builder.finish()
}

pub fn evaluate(program: &Program, input: &SeccompInput) -> Result<Action, InterpretError> {
    program.validate().map_err(InterpretError::InvalidProgram)?;
    let mut accumulator = 0_u32;
    let mut pc = 0_usize;

    while let Some(instruction) = program.instructions.get(pc) {
        match instruction.code {
            BPF_LD_W_ABS => {
                accumulator = load_word(input, instruction.k).ok_or(
                    InterpretError::InvalidProgram(ProgramError::InvalidLoadOffset {
                        index: pc,
                        offset: instruction.k,
                    }),
                )?;
                pc += 1;
            }
            BPF_JMP_JEQ_K => {
                let distance = if accumulator == instruction.k {
                    instruction.jt
                } else {
                    instruction.jf
                };
                pc += 1 + usize::from(distance);
            }
            BPF_RET_K => return Action::from_raw(instruction.k),
            code => {
                return Err(InterpretError::InvalidProgram(
                    ProgramError::UnsupportedInstruction(pc, code),
                ));
            }
        }
    }
    Err(InterpretError::NoReturn)
}

/// Evaluates a kernel-style filter stack. `programs` must be newest first; equal-precedence
/// results retain the newest filter's data, which is required for steady `clone3 -> ENOSYS` to
/// supersede bootstrap `clone3 -> EPERM`.
pub fn evaluate_stack_newest_first(
    programs: &[&Program],
    input: &SeccompInput,
) -> Result<Action, InterpretError> {
    let mut selected = Action::Allow;
    for program in programs {
        let candidate = evaluate(program, input)?;
        if candidate.precedence() > selected.precedence() {
            selected = candidate;
        }
    }
    Ok(selected)
}

#[derive(Debug)]
pub enum InstallError {
    UnsupportedTarget,
    InvalidProgram(ProgramError),
    ProgramLength(usize),
    Kernel(std::io::Error),
}

impl fmt::Display for InstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTarget => formatter
                .write_str("seccomp installation is supported only on aarch64-unknown-linux-musl"),
            Self::InvalidProgram(error) => write!(formatter, "invalid seccomp program: {error}"),
            Self::ProgramLength(len) => write!(formatter, "invalid sock_fprog length {len}"),
            Self::Kernel(error) => write!(formatter, "seccomp syscall failed: {error}"),
        }
    }
}

impl std::error::Error for InstallError {}

#[cfg(all(
    target_os = "linux",
    target_arch = "aarch64",
    target_env = "musl",
    target_vendor = "unknown"
))]
mod runtime_install {
    use super::{aarch64, InstallError, Instruction, Program, SECCOMP_SET_MODE_FILTER};
    use std::os::raw::c_long;

    #[repr(C)]
    struct SockFprog {
        len: u16,
        filter: *mut Instruction,
    }

    unsafe extern "C" {
        fn syscall(number: c_long, ...) -> c_long;
    }

    pub(super) fn install(program: &Program) -> Result<(), InstallError> {
        program.validate().map_err(InstallError::InvalidProgram)?;
        let len =
            u16::try_from(program.len()).map_err(|_| InstallError::ProgramLength(program.len()))?;
        let mut kernel_program = SockFprog {
            len,
            filter: program.instructions().as_ptr().cast_mut(),
        };
        // SAFETY: `kernel_program` and its instruction slice live through the syscall; the kernel
        // copies a validated classic-BPF program and does not retain either userspace pointer.
        let result = unsafe {
            syscall(
                aarch64::NR_SECCOMP as c_long,
                SECCOMP_SET_MODE_FILTER as c_long,
                0 as c_long,
                &mut kernel_program as *mut SockFprog,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(InstallError::Kernel(std::io::Error::last_os_error()))
        }
    }
}

/// Installs an already-derived program only on the pinned guest target.  This deliberately does
/// not set `no_new_privs`, choose a filter, or claim the surrounding containment ordering.
pub fn install_program(program: &Program) -> Result<(), InstallError> {
    #[cfg(all(
        target_os = "linux",
        target_arch = "aarch64",
        target_env = "musl",
        target_vendor = "unknown"
    ))]
    {
        runtime_install::install(program)
    }
    #[cfg(not(all(
        target_os = "linux",
        target_arch = "aarch64",
        target_env = "musl",
        target_vendor = "unknown"
    )))]
    {
        let _ = program;
        Err(InstallError::UnsupportedTarget)
    }
}
