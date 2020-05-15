use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::hashes::sha256d::Hash;
use bitcoin::Txid;
use branch_and_bound::branch_and_bound::{BranchAndBound, OutPointValue};

fn main() {
    let mut mandatory_utxos = Vec::new();
    mandatory_utxos.push(OutPointValue(
        OutPoint {
            txid: Txid::from(Hash::default()),
            vout: 0,
        },
        2,
    ));

    let mut optional_utxos = Vec::new();
    optional_utxos.push(OutPointValue(
        OutPoint {
            txid: Txid::from(Hash::default()),
            vout: 0,
        },
        9,
    ));
    optional_utxos.push(OutPointValue(
        OutPoint {
            txid: Txid::from(Hash::default()),
            vout: 0,
        },
        4,
    ));
    optional_utxos.push(OutPointValue(
        OutPoint {
            txid: Txid::from(Hash::default()),
            vout: 0,
        },
        5,
    ));
    optional_utxos.push(OutPointValue(
        OutPoint {
            txid: Txid::from(Hash::default()),
            vout: 0,
        },
        16,
    ));

    let b = BranchAndBound {
        spending_target: 27,
        mandatory_utxos: mandatory_utxos,
        optional_utxos: optional_utxos,
        addressees_num: 1,
        estimated_fees: 0,
        size_of_header: 0,
        size_per_output: 0,
        size_per_input: 0,
        tries: 100000,
    };

    let coins = b.select_coins().unwrap();
    for coin in coins {
        println!("{:?}", coin.1);
    }
}
