near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    CONTRACT_WASM_BYTES => "out/main.wasm",
}

use near_sdk_sim::init_simulator;
use near_sdk_sim::to_yocto;
use near_sdk_sim::UserAccount;
use near_sdk_sim::STORAGE_AMOUNT;

const CONTRACT_ID: &str = "contract";

pub fn init() -> (UserAccount, UserAccount, UserAccount) {
    // Use `None` for default genesis configuration; more info below
    let root = init_simulator(None);

    let contract = root.deploy(
        &CONTRACT_WASM_BYTES,
        CONTRACT_ID.to_string(),
        STORAGE_AMOUNT, // attached deposit
    );

    let alice = root.create_user(
        "alice".to_string(),
        to_yocto("100"), // initial balance
    );

    (root, contract, alice)
}

pub fn to_gas(tera_gas: &str) -> u64 {
    let part: Vec<_> = tera_gas.split('.').collect();
    let number = part[0].parse::<u64>().unwrap() * u64::pow(10, 12);
    if part.len() > 1 {
        let power = part[1].len() as u32;
        let mantissa = part[1].parse::<u64>().unwrap() * u64::pow(10, 12 - power);
        number + mantissa
    } else {
        number
    }
}
