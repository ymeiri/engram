//! Provider-free integrity check for the Stage C2B2a host foundation.

#[allow(dead_code)]
#[path = "../native_c2b2a.rs"]
mod native_c2b2a;

fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "native-c2b2a is a no-argument foundation check"
    );
    native_c2b2a::validate_compiled_foundation()
        .expect("compiled Stage C2B2a host foundation must be internally consistent");

    println!("foundation_valid=true");
    println!("provider_free=true");
    println!("runtime_implemented=false");
    println!("c2b2a_acceptance_proven=false");
    println!("flagship_goal_complete=false");
}
