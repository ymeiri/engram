#[cfg(feature = "supervisor")]
compile_error!("the C2B2a collector must be built without the supervisor feature");

use engram_native_c2b2a_payload::seccomp::{
    evaluate, steady_program, Action, SeccompInput, PTHREAD_CLONE_FLAGS,
};

const FOUNDATION_INCOMPLETE_EXIT_CODE: i32 = 78;
const FOUNDATION_SELF_CHECK_FAILED_EXIT_CODE: i32 = 70;

fn foundation_self_check() -> bool {
    let Ok(steady) = steady_program() else {
        return false;
    };
    let mut clone_args = [0_u64; 6];
    clone_args[0] = PTHREAD_CLONE_FLAGS;
    steady.validate().is_ok()
        && evaluate(
            &steady,
            &SeccompInput::aarch64(
                engram_native_c2b2a_payload::seccomp::aarch64::NR_CLONE,
                clone_args,
            ),
        ) == Ok(Action::Allow)
}

fn main() {
    // This is not a collector runtime.  It deliberately touches no fixture, Core, RocksDB,
    // descriptor, signal, or seccomp state until the complete frozen entry sequence exists.
    let exit_code = if foundation_self_check() {
        FOUNDATION_INCOMPLETE_EXIT_CODE
    } else {
        FOUNDATION_SELF_CHECK_FAILED_EXIT_CODE
    };
    std::process::exit(exit_code);
}
