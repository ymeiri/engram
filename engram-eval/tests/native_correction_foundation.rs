use engram_eval::native_correction::{
    NATIVE_CORRECTION_FAMILY, NATIVE_CORRECTION_PROTOCOL_SCHEMA_VERSION,
};

#[test]
fn native_correction_is_reachable_through_the_production_library() {
    assert_eq!(NATIVE_CORRECTION_FAMILY, "native_correction_v1");
    assert_eq!(NATIVE_CORRECTION_PROTOCOL_SCHEMA_VERSION, 2);
}
