#[path = "validation_contracts/mod.rs"]
mod validation_contracts;

#[test]
#[ignore = "invoked explicitly by scripts/run_validation_contract_matrix.sh"]
fn test_validation_contract_matrix() {
    if let Err(error) = validation_contracts::run() {
        panic!("{error}");
    }
}
