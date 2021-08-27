use crate::utils::init;
use crate::utils::to_gas;
use near_sdk_sim::DEFAULT_GAS;

#[test]
fn default() {
    let (root, contract, _alice) = init();

    let result = root.call(
        contract.account_id(),
        "decrement",
        &[].to_vec(),
        DEFAULT_GAS,
        0, // deposit
    );

    println!(
        "burnt tokens: {:.04}Ⓝ gas: {:.01} TeraGas",
        (result.tokens_burnt()) as f64 / 1e24,
        (result.gas_burnt()) as f64 / 1e12,
    );

    assert!(result.gas_burnt() <= to_gas("2.7"));

    let actual: i8 = root
        .view(contract.account_id(), "get_num", &[].to_vec())
        .unwrap_json();

    assert_eq!(-1, actual);
}
