use std::path::Path;

use circuitsim_engine::middle_end::serialize::CircuitFile;

const LATCHES: &str = include_str!("serialize/latches.sim");

#[test]
fn parse_test_sim() {
    let parsed = CircuitFile::from_sim(LATCHES)
        .expect("Unable to deserialize");

    assert_eq!(parsed.version, "1.9.1 2110 version");
    assert_eq!(parsed.global_bitsize, 1);
    assert_eq!(parsed.clock_speed, 1);
    assert!(!parsed.circuits.is_empty());
    assert!(!parsed.revision_signatures.is_empty());
}

#[test]
fn parse_test_sim_then_serialize() {
    let parsed = CircuitFile::from_sim(LATCHES)
        .expect("Unable to deserialize");
    let serialized = parsed.to_sim().expect("Unable to serialize");
    let reparsed = CircuitFile::from_sim(&serialized).expect("Unable to deserialize");

    assert_eq!(parsed, reparsed);
}

#[test]
fn read_test_sim() {
    let parsed = CircuitFile::from_sim(LATCHES)
        .expect("Unable to deserialize");

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/serialize/latches.sim");
    let loaded = CircuitFile::read_sim_file(&path).expect("Unable to read");

    assert_eq!(parsed, loaded);
}
