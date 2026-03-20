use std::path::Path;

use circuitsim_engine::middle_end::serialize::{CircuitFile, SerdeError};

const LATCHES: &str = include_str!("serialize/latches.sim");

#[test]
fn parse_test_sim() -> Result<(), SerdeError> {
    let parsed = CircuitFile::from_sim(LATCHES)?;

    assert_eq!(parsed.version, "1.9.1 2110 version");
    assert_eq!(parsed.global_bitsize, 1);
    assert_eq!(parsed.clock_speed, 1);
    assert!(!parsed.circuits.is_empty());
    assert!(!parsed.revision_signatures.is_empty());
    Ok(())
}

#[test]
fn parse_test_sim_then_serialize() -> Result<(), SerdeError> {
    let parsed = CircuitFile::from_sim(LATCHES)?;
    let serialized = parsed.to_sim()?;
    let reparsed = CircuitFile::from_sim(&serialized)?;

    assert_eq!(parsed, reparsed);
    Ok(())
}

#[test]
fn read_test_sim() -> Result<(), SerdeError> {
    let parsed = CircuitFile::from_sim(LATCHES)?;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/serialize/latches.sim");
    let loaded = CircuitFile::read_sim_file(&path)?;

    assert_eq!(parsed, loaded);
    Ok(())
}

const SIMPLE_AND: &str = include_str!("serialize/simple_and.sim");

#[test]
fn parse_test_sim_simple_and() -> Result<(), SerdeError> {
    let parsed = CircuitFile::from_sim(SIMPLE_AND)?;

    assert_eq!(parsed.version, "1.11.2-CE");
    assert_eq!(parsed.global_bitsize, 1);
    assert_eq!(parsed.clock_speed, 1);
    assert!(!parsed.circuits.is_empty());
    assert!(!parsed.revision_signatures.is_empty());
    Ok(())
}
#[test]
fn read_test_sim_simple_and() -> Result<(), SerdeError> {
    let parsed = CircuitFile::from_sim(SIMPLE_AND)?;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/serialize/simple_and.sim");
    let loaded = CircuitFile::read_sim_file(&path)?;

    assert_eq!(parsed, loaded);
    Ok(())
}
