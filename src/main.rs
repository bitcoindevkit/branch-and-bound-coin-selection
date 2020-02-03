use bitcoin::blockdata::script::Script;
use bitcoin::blockdata::transaction::TxOut;

use rand::{thread_rng, Rng};
use std::convert::TryInto;

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

struct BranchAndBound<'a> {
    spending_target: u64,
    available_utxos: Vec<&'a TxOut>,
    addressees_num: u64,
    estimated_fees: u64,
    size_of_header: u64,
    size_per_output: u64,
    size_per_input: u64,
    tries: u64,
}

#[derive(Debug)]
enum Error {
    InsufficientFunds,
}

impl<'a> BranchAndBound<'a> {
    fn select_coins(mut self) -> Result<Vec<&'a TxOut>, Error> {
        self.available_utxos.sort_by(|a, b| b.value.cmp(&a.value));
        let mut selected_coins = Vec::new();
        let result = self.branch_and_bound(0, &mut selected_coins, 0);

        if result {
            Ok(selected_coins)
        } else {
            // If no match, Single Random Draw
            self.single_random_draw()
        }
    }

    fn single_random_draw(mut self) -> Result<Vec<&'a TxOut>, Error> {
        thread_rng().shuffle(&mut self.available_utxos);
        let mut sum = 0;

        let mut target = self.spending_target + self.output_cost() + self.header_cost();

        let cost_per_input = self.input_cost(1);
        let selected_coins = self
            .available_utxos
            .into_iter()
            .take_while(|x| {
                sum += x.value;
                target += cost_per_input;
                sum - x.value < target
            })
            .collect();

        if sum >= target {
            Ok(selected_coins)
        } else {
            Err(Error::InsufficientFunds)
        }
    }

    fn branch_and_bound(
        &mut self,
        depth: usize,
        current_selection: &mut Vec<&'a TxOut>,
        effective_value: u64,
    ) -> bool {
        let input_cost = self.input_cost(current_selection.len().try_into().unwrap());
        let output_cost = self.output_cost();
        let target_for_match = self.spending_target + self.header_cost() + output_cost + input_cost;
        let match_range = input_cost + output_cost;

        if effective_value > target_for_match + match_range {
            return false;
        }

        if effective_value >= target_for_match {
            return true;
        }

        if self.tries <= 0 || depth >= self.available_utxos.len() {
            return false;
        }

        self.tries = self.tries - 1;

        // Exploring omission and inclusion branch
        let current_utxo_value = self.available_utxos[depth].value;
        current_selection.push(self.available_utxos[depth]);

        if self.branch_and_bound(
            depth + 1,
            current_selection,
            effective_value + current_utxo_value,
        ) {
            return true;
        }

        current_selection.pop();

        self.branch_and_bound(depth + 1, current_selection, effective_value)
    }

    fn input_cost(&self, input_num: u64) -> u64 {
        self.size_per_input * self.estimated_fees * input_num
    }

    fn output_cost(&self) -> u64 {
        self.addressees_num * self.size_per_output * self.estimated_fees
    }

    fn header_cost(&self) -> u64 {
        self.size_of_header * self.estimated_fees
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use rand::{Rng, SeedableRng, StdRng};

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
