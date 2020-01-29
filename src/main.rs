use bitcoin::blockdata::script::Script;
use bitcoin::blockdata::transaction::TxOut;

use std::convert::TryInto;

const DUST_LIMIT: u64 = 540;

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
        available_utxos: available_utxos,
        minimum_change: None,
        addressees_num: 1,
        estimated_fees: 0,
        cost_of_header: 0,
        size_per_output: 0,
        size_per_input: 0,
        tries: 100000,
    };

    println!("{:?}", b.select_coins());
}

struct BranchAndBound {
    spending_target: u64,
    available_utxos: Vec<TxOut>,
    minimum_change: Option<u64>,
    addressees_num: u64,
    estimated_fees: u64,
    cost_of_header: u64,
    size_per_output: u64,
    size_per_input: u64,
    tries: u64,
}

impl BranchAndBound {
    fn select_coins(mut self) -> Vec<TxOut> {
        self.available_utxos.sort_by(|a, b| b.value.cmp(&a.value));
        let mut selected_coins = Vec::new();
        let result = self.search(0, &mut selected_coins, 0);

        // If no match, Single Random Draw
        if !result {
            //thread_rng().shuffle(&mut self.available_utxos);
            let mut sum = 0;

            let mut target = match self.minimum_change {
                Some(x) if x > DUST_LIMIT => self.spending_target + self.output_cost() + x,
                _ => self.spending_target + self.output_cost(),
            };

            let cost_per_input = self.input_cost(1);
            self.available_utxos
                .into_iter()
                .take_while(|x| {
                    sum += x.value;
                    target += cost_per_input;
                    sum - x.value < target
                })
                .collect()
        } else {
            selected_coins
        }
    }

    fn search(
        &mut self,
        depth: usize,
        current_selection: &mut Vec<TxOut>,
        effective_value: u64,
    ) -> bool {
        let input_cost = self.input_cost(current_selection.len().try_into().unwrap());
        let output_cost = self.output_cost();
        let target_for_match = self.spending_target + self.cost_of_header + output_cost;
        let match_range = input_cost + output_cost;

        if effective_value > target_for_match + match_range {
            return false;
        }

        if effective_value >= target_for_match {
            return true;
        }

        if self.tries == 0 || depth >= self.available_utxos.len() {
            return false;
        }

        self.tries = self.tries - 1;

        // Exploring omission and inclusion branch
        let current_utxo_value = self.available_utxos[depth].value;
        current_selection.push(self.available_utxos[depth].clone());

        if self.search(
            depth + 1,
            current_selection,
            effective_value + current_utxo_value,
        ) {
            return true;
        }

        current_selection.pop();

        self.search(depth + 1, current_selection, effective_value)
    }

    fn input_cost(&self, input_num: u64) -> u64 {
        self.size_per_input * self.estimated_fees * input_num
    }

    fn output_cost(&self) -> u64 {
        let output_num = match self.minimum_change {
            Some(x) if x > DUST_LIMIT => self.addressees_num + 1,
            _ => self.addressees_num,
        };
        output_num * self.size_per_output * self.estimated_fees
    }
}
