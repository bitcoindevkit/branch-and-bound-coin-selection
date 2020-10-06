pub mod branch_and_bound;

#[cfg(test)]
mod test {

    use crate::branch_and_bound::{BranchAndBound, OutPointValue};
    use bitcoin::blockdata::transaction::OutPoint;
    use bitcoin::hashes::sha256d::Hash;
    use bitcoin::Txid;
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use rand::{thread_rng, Rng, SeedableRng};

    fn generate_random_utxos(rng: &mut StdRng, utxos_number: i32) -> Vec<OutPointValue> {
        let mut optional_utxos = Vec::new();
        for _i in 0..utxos_number {
            optional_utxos.push(OutPointValue(
                OutPoint {
                    txid: Txid::from(Hash::default()),
                    vout: 0,
                },
                rng.gen_range(0, 2000),
            ));
        }
        optional_utxos
    }

    fn sum_random_utxos(rng: &mut StdRng, optional_utxos: &mut Vec<OutPointValue>) -> u64 {
        let utxos_picked_len = rng.gen_range(2, optional_utxos.len() / 2);
        optional_utxos.shuffle(&mut thread_rng());
        optional_utxos[..utxos_picked_len]
            .iter()
            .fold(0, |acc, x| acc + x.1)
    }

    #[test]
    fn test_exact_match() {
        // Exact matches, if they exist,
        // are always found when tries < 2^len(utxos)
        let seed = [0; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        for _i in 0..200 {
            let mut optional_utxos = generate_random_utxos(&mut rng, 30);
            let mandatory_utxos = Vec::new();
            let sum_utxos_picked = sum_random_utxos(&mut rng, &mut optional_utxos);
            let b = BranchAndBound {
                spending_target: sum_utxos_picked,
                optional_utxos,
                mandatory_utxos,
                addressees_num: 1,
                estimated_fee_rate: 0,
                size_of_header: 0,
                size_per_output: 0,
                size_per_input: 0,
                tries: 100000,
            };
            assert_eq!(
                b.select_coins()
                    .unwrap()
                    .into_iter()
                    .fold(0, |acc, x| acc + x.1),
                sum_utxos_picked
            );
        }
    }

    #[test]
    fn test_maybe_exact_match() {
        let seed = [0; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        for _i in 0..200 {
            let mut optional_utxos = generate_random_utxos(&mut rng, 1000);
            let mandatory_utxos = Vec::new();
            let spending_target = sum_random_utxos(&mut rng, &mut optional_utxos);
            let addressees_num = 3;
            let estimated_fee_rate = 20;
            let size_of_header = 10;
            let size_per_output = 10;
            let size_per_input = 5;
            let b = BranchAndBound {
                spending_target,
                optional_utxos,
                mandatory_utxos,
                addressees_num,
                estimated_fee_rate,
                size_of_header,
                size_per_output,
                size_per_input,
                tries: 100000,
            };
            let selected_coins = b.select_coins().unwrap();
            let target = spending_target
                + estimated_fee_rate
                    * (addressees_num * size_per_output
                        + (selected_coins.len() as u64) * size_per_input
                        + size_of_header);
            assert!(selected_coins.into_iter().fold(0, |acc, x| acc + x.1) >= target);
        }
    }

    #[test]
    #[should_panic]
    fn test_insufficient_funds() {
        let seed = [0; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let optional_utxos = generate_random_utxos(&mut rng, 5);
        let mandatory_utxos = Vec::new();
        let sum_utxos_picked = optional_utxos.iter().fold(0, |acc, x| acc + x.1);
        let b = BranchAndBound {
            spending_target: sum_utxos_picked,
            optional_utxos: optional_utxos,
            mandatory_utxos,
            addressees_num: rng.gen_range(1, 100),
            estimated_fee_rate: rng.gen_range(1, 200),
            size_of_header: rng.gen_range(1, 200),
            size_per_output: rng.gen_range(1, 200),
            size_per_input: rng.gen_range(1, 200),
            tries: 100000,
        };

        b.select_coins().unwrap();
    }

    #[test]
    fn test_mandatory() {
        // Exact matches, if they exist,
        // are always found when tries < 2^len(utxos)
        let seed = [0; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        for _i in 0..200 {
            let mut optional_utxos = generate_random_utxos(&mut rng, 30);
            let mandatory_utxos = generate_random_utxos(&mut rng, 3);
            let sum_utxos_picked = sum_random_utxos(&mut rng, &mut optional_utxos);
            let b = BranchAndBound {
                spending_target: sum_utxos_picked,
                optional_utxos,
                mandatory_utxos: mandatory_utxos.clone(),
                addressees_num: 1,
                estimated_fee_rate: 0,
                size_of_header: 0,
                size_per_output: 0,
                size_per_input: 0,
                tries: 100000,
            };
            let utxos_selected = b.select_coins().unwrap();

            for utxo in mandatory_utxos {
                assert!(utxos_selected.contains(&utxo));
            }

            assert!(utxos_selected.into_iter().fold(0, |acc, x| acc + x.1) >= sum_utxos_picked);
        }
    }
}
