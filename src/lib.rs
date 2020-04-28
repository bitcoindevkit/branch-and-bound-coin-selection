pub mod branch_and_bound;

#[cfg(test)]
mod test {

    use crate::branch_and_bound::BranchAndBound;
    use bitcoin::blockdata::transaction::TxOut;
    use bitcoin::blockdata::script::Script;
    use rand::{thread_rng, SeedableRng, StdRng, Rng};


    fn generate_random_utxos(rng: &mut StdRng, utxos_number: i32) -> Vec<TxOut> {
        let mut available_utxos = Vec::new();
        for _i in 0..utxos_number {
            available_utxos.push(TxOut {
                value: rng.gen_range(0, 2000),
                script_pubkey: Script::new(),
            });
        }
        available_utxos
    }

    fn sum_random_utxos<'a>(rng: &mut StdRng, available_utxos: &mut Vec<&'a TxOut>) -> u64 {
        let utxos_picked_len = rng.gen_range(2, available_utxos.len() / 2);
        thread_rng().shuffle(available_utxos);
        available_utxos[..utxos_picked_len]
            .iter()
            .fold(0, |acc, x| acc + x.value)
    }

    #[test]
    fn test_exact_match() {
        // Exact matches are always found when tries < 2^len(utxos)
        let seed: &[_] = &[1, 2, 3, 4];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        for _i in 0..200 {
            let available_utxos = generate_random_utxos(&mut rng, 30);
            let mut available_utxos = available_utxos.iter().collect();
            let sum_utxos_picked = sum_random_utxos(&mut rng, &mut available_utxos);
            let b = BranchAndBound {
                spending_target: sum_utxos_picked,
                available_utxos,
                addressees_num: 1,
                estimated_fees: 0,
                size_of_header: 0,
                size_per_output: 0,
                size_per_input: 0,
                tries: 100000,
            };
            assert_eq!(
                b.select_coins()
                    .unwrap()
                    .into_iter()
                    .fold(0, |acc, x| acc + x.value),
                sum_utxos_picked
            );
        }
    }

    #[test]
    fn test_maybe_exact_match() {
        let seed: &[_] = &[1, 2, 3, 4];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        for _i in 0..200 {
            let available_utxos = generate_random_utxos(&mut rng, 1000);
            let mut available_utxos = available_utxos.iter().collect();
            let spending_target = sum_random_utxos(&mut rng, &mut available_utxos);
            let addressees_num = 3;
            let estimated_fees = 20;
            let size_of_header = 10;
            let size_per_output = 10;
            let size_per_input = 5;
            let b = BranchAndBound {
                spending_target,
                available_utxos,
                addressees_num,
                estimated_fees,
                size_of_header,
                size_per_output,
                size_per_input,
                tries: 100000,
            };
            let selected_coins = b.select_coins().unwrap();
            let target = spending_target
                + estimated_fees
                    * (addressees_num * size_per_output
                        + (selected_coins.len() as u64) * size_per_input
                        + size_of_header);
            assert!(selected_coins.into_iter().fold(0, |acc, x| acc + x.value) >= target);
        }
    }

    #[test]
    #[should_panic]
    fn test_insufficient_funds() {
        let seed: &[_] = &[1, 2, 3, 4];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let available_utxos = generate_random_utxos(&mut rng, 5);
        let sum_utxos_picked = available_utxos.iter().fold(0, |acc, x| acc + x.value);
        let b = BranchAndBound {
            spending_target: sum_utxos_picked,
            available_utxos: available_utxos.iter().collect(),
            addressees_num: rng.gen_range(1, 100),
            estimated_fees: rng.gen_range(1, 200),
            size_of_header: rng.gen_range(1, 200),
            size_per_output: rng.gen_range(1, 200),
            size_per_input: rng.gen_range(1, 200),
            tries: 100000,
        };

        b.select_coins().unwrap();
    }
}