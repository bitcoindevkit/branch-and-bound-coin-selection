use bitcoin::blockdata::script::Script;
use bitcoin::blockdata::transaction::TxOut;
use branch_and_bound::branch_and_bound::BranchAndBound;

fn main() {
    let mut available_utxos = Vec::new();
    available_utxos.push(TxOut {
        value: 9,
        script_pubkey: Script::new(),
    });
    available_utxos.push(TxOut {
        value: 4,
        script_pubkey: Script::new(),
    });
    available_utxos.push(TxOut {
        value: 5,
        script_pubkey: Script::new(),
    });
    available_utxos.push(TxOut {
        value: 16,
        script_pubkey: Script::new(),
    });

    let b = BranchAndBound {
        spending_target: 26,
        available_utxos: available_utxos.iter().collect(),
        addressees_num: 1,
        estimated_fees: 0,
        size_of_header: 0,
        size_per_output: 0,
        size_per_input: 0,
        tries: 100000,
    };

    println!("{:?}", b.select_coins());
}
