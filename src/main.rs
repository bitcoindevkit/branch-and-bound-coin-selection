use bitcoin::blockdata::script::Script;
use bitcoin::blockdata::transaction::TxOut;
use branch_and_bound::branch_and_bound::BranchAndBound;

fn main() {
    let mut mandatory_utxos = Vec::new();
    mandatory_utxos.push(TxOut {
        value: 23,
        script_pubkey: Script::new(),
    });

    let mut optional_utxos = Vec::new();
    optional_utxos.push(TxOut {
        value: 9,
        script_pubkey: Script::new(),
    });
    optional_utxos.push(TxOut {
        value: 4,
        script_pubkey: Script::new(),
    });
    optional_utxos.push(TxOut {
        value: 5,
        script_pubkey: Script::new(),
    });
    optional_utxos.push(TxOut {
        value: 16,
        script_pubkey: Script::new(),
    });

    let b = BranchAndBound {
        spending_target: 26,
        mandatory_utxos: mandatory_utxos.iter().collect(),
        optional_utxos: optional_utxos.iter().collect(),
        addressees_num: 1,
        estimated_fees: 0,
        size_of_header: 0,
        size_per_output: 0,
        size_per_input: 0,
        tries: 100000,
    };

    println!("{:?}", b.select_coins());
}
