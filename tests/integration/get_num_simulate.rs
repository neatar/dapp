use crate::utils::init;

#[test]
fn default() {
    let (root, contract, _alice) = init();

    let actual: i8 = root
        .view(contract.account_id(), "get_num", &[].to_vec())
        .unwrap_json();

    assert_eq!(0, actual);
}
