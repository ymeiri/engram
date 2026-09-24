#[cfg(feature = "collector")]
compile_error!("the C2B2a supervisor must be built without the collector feature");

use engram_native_c2b2a_payload::seccomp::{
    bootstrap_program, evaluate, steady_program, Action, SeccompInput,
};

const FOUNDATION_INCOMPLETE_EXIT_CODE: i32 = 78;
const FOUNDATION_SELF_CHECK_FAILED_EXIT_CODE: i32 = 70;

fn foundation_self_check() -> bool {
    let Ok(bootstrap) = bootstrap_program() else {
        return false;
    };
    let Ok(steady) = steady_program() else {
        return false;
    };
    bootstrap.validate().is_ok()
        && steady.validate().is_ok()
        && evaluate(
            &bootstrap,
            &SeccompInput::aarch64(
                engram_native_c2b2a_payload::seccomp::aarch64::NR_READ,
                [0; 6],
            ),
        ) == Ok(Action::Allow)
}

fn main() {
    // This foundation intentionally performs no namespace, mount, cgroup, process, or seccomp
    // mutation.  Until the complete frozen supervisor lifecycle is implemented, every invocation
    // fails closed after checking that the embedded policy can be derived.
    let exit_code = if foundation_self_check() {
        FOUNDATION_INCOMPLETE_EXIT_CODE
    } else {
        FOUNDATION_SELF_CHECK_FAILED_EXIT_CODE
    };
    std::process::exit(exit_code);
}
