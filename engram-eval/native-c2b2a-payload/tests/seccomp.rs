use std::collections::BTreeSet;

use engram_native_c2b2a_payload::seccomp::{
    aarch64, bootstrap_program, evaluate, evaluate_stack_newest_first, steady_program, Action,
    SeccompInput, ABI_WRAPPERS_WITHOUT_SYSCALL_NUMBERS, AT_EMPTY_PATH, ENOSYS, EPERM,
    EXPECTED_EXECVEAT_FD, NETWORK_SYSCALLS, PR_GET_DUMPABLE, PR_GET_NAME, PR_GET_PDEATHSIG,
    PR_SET_DUMPABLE, PR_SET_NAME, PTHREAD_CLONE_FLAGS, SECCOMP_SET_MODE_FILTER,
    STEADY_ALLOWED_SYSCALLS,
};
use sha2::{Digest, Sha256};

const EXPECTED_STEADY_ALLOWED_AARCH64: &[u32] = &[
    63, 64, 65, 66, 67, 68, 57, 436, 23, 24, 25, 29, 62, 56, 79, 80, 291, 44, 61, 78, 48, 439, 34,
    35, 38, 276, 37, 46, 47, 82, 83, 84, 213, 223, 222, 226, 215, 216, 233, 232, 214, 278, 134,
    135, 139, 132, 98, 449, 96, 99, 293, 283, 124, 123, 122, 113, 115, 101, 169, 165, 153, 160,
    179, 172, 173, 178, 174, 175, 176, 177, 261, 166, 8, 9, 10, 5, 6, 7, 14, 15, 16, 20, 21, 22,
    441, 19, 85, 86, 87, 73, 72, 128, 131, 93, 94,
];
const EXPECTED_NETWORK_AARCH64: &[u32] = &[
    198, 199, 203, 200, 201, 202, 242, 206, 211, 269, 207, 212, 243, 204, 205, 208, 209, 210,
];

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn action(
    program: &engram_native_c2b2a_payload::seccomp::Program,
    nr: u32,
    args: [u64; 6],
) -> Action {
    evaluate(program, &SeccompInput::aarch64(nr, args)).expect("valid generated program")
}

#[test]
fn programs_are_deterministic_valid_classic_bpf_with_golden_prefix() {
    let bootstrap = bootstrap_program().expect("bootstrap builds");
    let steady = steady_program().expect("steady builds");
    bootstrap.validate().expect("bootstrap validates");
    steady.validate().expect("steady validates");

    assert_eq!(bootstrap, bootstrap_program().expect("second bootstrap"));
    assert_eq!(steady, steady_program().expect("second steady"));
    assert_eq!(bootstrap.canonical_bytes().len(), bootstrap.len() * 8);
    assert_eq!(steady.canonical_bytes().len(), steady.len() * 8);
    assert_eq!(bootstrap.len(), 293);
    assert_eq!(steady.len(), 255);
    assert!(bootstrap.len() < 4096);
    assert!(steady.len() < 4096);

    let golden_prefix = [
        0x20, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, // load arch
        0x15, 0x00, 0x01, 0x00, 0xb7, 0x00, 0x00, 0xc0, // require AArch64
        0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, // kill mismatch
        0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // load nr
    ];
    assert_eq!(
        &bootstrap.canonical_bytes()[..golden_prefix.len()],
        golden_prefix
    );
    assert_eq!(
        &steady.canonical_bytes()[..golden_prefix.len()],
        golden_prefix
    );
    assert_eq!(
        sha256_hex(&bootstrap.canonical_bytes()),
        "426f846455891b9142f4e0efc9c2ad30a238cbcd8b774cbec39f3c67c36d021b"
    );
    assert_eq!(
        sha256_hex(&steady.canonical_bytes()),
        "b5291fe17d82586986aaf58e53421bca49e9173c57ba8b0ccb8b5ec08ac83381"
    );
}

#[test]
fn architecture_mismatch_kills_before_syscall_rules() {
    for program in [bootstrap_program().unwrap(), steady_program().unwrap()] {
        let mut input = SeccompInput::aarch64(aarch64::NR_READ, [0; 6]);
        assert_eq!(evaluate(&program, &input).unwrap(), Action::Allow);

        input.arch = 0xc000_003e;
        assert_eq!(evaluate(&program, &input).unwrap(), Action::KillProcess);
        input.arch = 0;
        input.nr = aarch64::NR_SOCKET as i32;
        assert_eq!(evaluate(&program, &input).unwrap(), Action::KillProcess);
    }
}

#[test]
fn bootstrap_execveat_binds_full_fd_and_flags() {
    let bootstrap = bootstrap_program().unwrap();
    let mut args = [0_u64; 6];
    args[0] = EXPECTED_EXECVEAT_FD;
    args[1] = 0x1000;
    args[4] = AT_EMPTY_PATH;
    assert_eq!(
        action(&bootstrap, aarch64::NR_EXECVEAT, args),
        Action::Allow
    );

    // Classic BPF cannot inspect the pathname bytes.  Both pointers are accepted here; the
    // required zero byte is independently bound by the frozen source/ELF call-site firewall.
    args[1] = 0xfeed_beef;
    assert_eq!(
        action(&bootstrap, aarch64::NR_EXECVEAT, args),
        Action::Allow
    );

    for invalid_fd in [0, 6, 8, EXPECTED_EXECVEAT_FD | (1_u64 << 32)] {
        args[0] = invalid_fd;
        assert_eq!(
            action(&bootstrap, aarch64::NR_EXECVEAT, args),
            Action::Errno(EPERM)
        );
    }
    args[0] = EXPECTED_EXECVEAT_FD;
    for invalid_flags in [0, AT_EMPTY_PATH | 1, AT_EMPTY_PATH | (1_u64 << 32)] {
        args[4] = invalid_flags;
        assert_eq!(
            action(&bootstrap, aarch64::NR_EXECVEAT, args),
            Action::Errno(EPERM)
        );
    }

    assert_eq!(
        action(&bootstrap, aarch64::NR_EXECVE, [0; 6]),
        Action::Errno(EPERM)
    );
    let steady = steady_program().unwrap();
    args[4] = AT_EMPTY_PATH;
    assert_eq!(
        action(&steady, aarch64::NR_EXECVEAT, args),
        Action::Errno(EPERM)
    );
}

#[test]
fn every_network_syscall_returns_eperm_in_both_programs() {
    assert_eq!(NETWORK_SYSCALLS.len(), 18);
    assert_eq!(
        NETWORK_SYSCALLS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        NETWORK_SYSCALLS.len()
    );
    for program in [bootstrap_program().unwrap(), steady_program().unwrap()] {
        for &nr in NETWORK_SYSCALLS {
            assert_eq!(action(&program, nr, [0; 6]), Action::Errno(EPERM));
        }
        assert_eq!(
            action(&program, aarch64::NR_SOCKET - 1, [0; 6]),
            Action::Trap
        );
    }
}

#[test]
fn clone3_and_process_spawn_fail_with_frozen_errno() {
    let bootstrap = bootstrap_program().unwrap();
    let steady = steady_program().unwrap();

    assert_eq!(
        action(&bootstrap, aarch64::NR_CLONE3, [0; 6]),
        Action::Errno(EPERM)
    );
    assert_eq!(
        action(&steady, aarch64::NR_CLONE3, [0; 6]),
        Action::Errno(ENOSYS)
    );
    assert_eq!(
        action(&steady, aarch64::NR_EXECVE, [0; 6]),
        Action::Errno(EPERM)
    );

    // AArch64 has no fork/vfork syscall numbers.  Musl expresses them as non-pthread clone
    // tuples, and both exact semantic forms are rejected by the clone rule.
    for semantic_spawn_flags in [17_u64, 0x0000_4111] {
        let mut args = [0_u64; 6];
        args[0] = semantic_spawn_flags;
        assert_eq!(
            action(&steady, aarch64::NR_CLONE, args),
            Action::Errno(EPERM)
        );
    }
    assert_eq!(
        ABI_WRAPPERS_WITHOUT_SYSCALL_NUMBERS,
        &["access", "fork", "vfork"]
    );
}

#[test]
fn only_the_complete_pinned_pthread_clone_mask_is_allowed() {
    for program in [bootstrap_program().unwrap(), steady_program().unwrap()] {
        let mut args = [0_u64; 6];
        args[0] = PTHREAD_CLONE_FLAGS;
        args[1] = 0x1234;
        args[2] = 0x5678;
        assert_eq!(action(&program, aarch64::NR_CLONE, args), Action::Allow);

        for bit in 0..64 {
            args[0] = PTHREAD_CLONE_FLAGS ^ (1_u64 << bit);
            assert_eq!(
                action(&program, aarch64::NR_CLONE, args),
                Action::Errno(EPERM),
                "bit {bit} was not bound"
            );
        }
    }
    assert_eq!(PTHREAD_CLONE_FLAGS & 0xff, 0, "zero exit signal");
    assert_eq!(PTHREAD_CLONE_FLAGS & 0x4000, 0, "no CLONE_VFORK");
    assert_eq!(PTHREAD_CLONE_FLAGS >> 32, 0, "zero high word");
}

#[test]
fn prctl_is_limited_to_the_phase_specific_exact_options() {
    let bootstrap = bootstrap_program().unwrap();
    let steady = steady_program().unwrap();

    for option in [PR_SET_NAME, PR_GET_NAME] {
        let mut args = [0_u64; 6];
        args[0] = option;
        assert_eq!(action(&steady, aarch64::NR_PRCTL, args), Action::Allow);
    }
    for option in 0_u64..=64 {
        if option != PR_SET_NAME && option != PR_GET_NAME {
            let mut args = [0_u64; 6];
            args[0] = option;
            assert_eq!(action(&steady, aarch64::NR_PRCTL, args), Action::Trap);
        }
    }
    let mut args = [0_u64; 6];
    args[0] = PR_SET_NAME | (1_u64 << 32);
    assert_eq!(action(&steady, aarch64::NR_PRCTL, args), Action::Trap);

    for option in [PR_GET_PDEATHSIG, PR_GET_DUMPABLE, PR_SET_NAME, PR_GET_NAME] {
        args = [0; 6];
        args[0] = option;
        assert_eq!(action(&bootstrap, aarch64::NR_PRCTL, args), Action::Allow);
    }
    args = [0; 6];
    args[0] = PR_SET_DUMPABLE;
    assert_eq!(action(&bootstrap, aarch64::NR_PRCTL, args), Action::Allow);
    for invalid_value in [1, 1_u64 << 32] {
        args[1] = invalid_value;
        assert_eq!(action(&bootstrap, aarch64::NR_PRCTL, args), Action::Trap);
    }
}

#[test]
fn bootstrap_allows_only_the_exact_steady_filter_install_tuple() {
    let bootstrap = bootstrap_program().unwrap();
    let steady = steady_program().unwrap();
    let mut args = [0_u64; 6];
    args[0] = SECCOMP_SET_MODE_FILTER;
    args[1] = 0;
    args[2] = 0xfeed_beef;
    assert_eq!(action(&bootstrap, aarch64::NR_SECCOMP, args), Action::Allow);
    assert_eq!(action(&steady, aarch64::NR_SECCOMP, args), Action::Trap);

    for (operation, flags) in [(0, 0), (2, 0), (SECCOMP_SET_MODE_FILTER, 1)] {
        args[0] = operation;
        args[1] = flags;
        assert_eq!(action(&bootstrap, aarch64::NR_SECCOMP, args), Action::Trap);
    }
}

#[test]
fn complete_steady_allowlist_is_unique_and_allowed() {
    let bootstrap = bootstrap_program().unwrap();
    let steady = steady_program().unwrap();
    assert_eq!(STEADY_ALLOWED_SYSCALLS.len(), 95);
    assert_eq!(
        STEADY_ALLOWED_SYSCALLS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        STEADY_ALLOWED_SYSCALLS.len()
    );
    for &nr in STEADY_ALLOWED_SYSCALLS {
        assert_eq!(action(&bootstrap, nr, [0; 6]), Action::Allow, "nr={nr}");
        assert_eq!(action(&steady, nr, [0; 6]), Action::Allow, "nr={nr}");
    }

    for trapped in [
        aarch64::NR_MOUNT,
        aarch64::NR_UMOUNT2,
        aarch64::NR_PIVOT_ROOT,
        aarch64::NR_CHROOT,
        aarch64::NR_UNSHARE,
        aarch64::NR_BPF,
        u32::MAX,
    ] {
        assert_eq!(action(&steady, trapped, [0; 6]), Action::Trap);
    }
}

#[test]
fn independent_aarch64_oracle_covers_the_complete_bounded_syscall_domain() {
    let bootstrap = bootstrap_program().unwrap();
    let steady = steady_program().unwrap();
    assert_eq!(EXPECTED_EXECVEAT_FD, 7);
    assert_eq!(AT_EMPTY_PATH, 0x1000);
    assert_eq!(PTHREAD_CLONE_FLAGS, 0x007d_0f00);
    assert_eq!(PR_GET_PDEATHSIG, 2);
    assert_eq!(PR_GET_DUMPABLE, 3);
    assert_eq!(PR_SET_DUMPABLE, 4);
    assert_eq!(PR_SET_NAME, 15);
    assert_eq!(PR_GET_NAME, 16);
    assert_eq!(SECCOMP_SET_MODE_FILTER, 1);
    assert_eq!(STEADY_ALLOWED_SYSCALLS, EXPECTED_STEADY_ALLOWED_AARCH64);
    assert_eq!(NETWORK_SYSCALLS, EXPECTED_NETWORK_AARCH64);

    for nr in (0_u32..=1024).chain(std::iter::once(u32::MAX)) {
        let steady_expected = if EXPECTED_STEADY_ALLOWED_AARCH64.contains(&nr) {
            Action::Allow
        } else if EXPECTED_NETWORK_AARCH64.contains(&nr) || matches!(nr, 220 | 221 | 281) {
            Action::Errno(EPERM)
        } else if nr == 435 {
            Action::Errno(ENOSYS)
        } else {
            Action::Trap
        };
        assert_eq!(
            action(&steady, nr, [0; 6]),
            steady_expected,
            "steady nr={nr}"
        );

        let bootstrap_expected = if EXPECTED_STEADY_ALLOWED_AARCH64.contains(&nr) {
            Action::Allow
        } else if EXPECTED_NETWORK_AARCH64.contains(&nr) || matches!(nr, 220 | 221 | 281 | 435) {
            Action::Errno(EPERM)
        } else {
            Action::Trap
        };
        assert_eq!(
            action(&bootstrap, nr, [0; 6]),
            bootstrap_expected,
            "bootstrap nr={nr}"
        );
    }
}

#[test]
fn newest_filter_wins_equal_errno_precedence_in_the_stack() {
    let bootstrap = bootstrap_program().unwrap();
    let steady = steady_program().unwrap();
    let filters = [&steady, &bootstrap];

    let clone3 = SeccompInput::aarch64(aarch64::NR_CLONE3, [0; 6]);
    assert_eq!(
        evaluate_stack_newest_first(&filters, &clone3).unwrap(),
        Action::Errno(ENOSYS)
    );

    let mut execveat_args = [0_u64; 6];
    execveat_args[0] = EXPECTED_EXECVEAT_FD;
    execveat_args[4] = AT_EMPTY_PATH;
    let execveat = SeccompInput::aarch64(aarch64::NR_EXECVEAT, execveat_args);
    assert_eq!(
        evaluate_stack_newest_first(&filters, &execveat).unwrap(),
        Action::Errno(EPERM)
    );
    let read = SeccompInput::aarch64(aarch64::NR_READ, [0; 6]);
    assert_eq!(
        evaluate_stack_newest_first(&filters, &read).unwrap(),
        Action::Allow
    );
}

#[cfg(not(all(
    target_os = "linux",
    target_arch = "aarch64",
    target_env = "musl",
    target_vendor = "unknown"
)))]
#[test]
fn installer_fails_closed_off_the_pinned_guest_target() {
    use engram_native_c2b2a_payload::seccomp::{install_program, InstallError};

    let program = steady_program().unwrap();
    assert!(matches!(
        install_program(&program),
        Err(InstallError::UnsupportedTarget)
    ));
}
