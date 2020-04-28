use bitcoin::blockdata::transaction::TxOut;

use rand::{thread_rng, Rng};
use std::convert::TryInto;

pub struct BranchAndBound<'a> {
    pub spending_target: u64,
    pub mandatory_utxos: Vec<&'a TxOut>,
    pub optional_utxos: Vec<&'a TxOut>,
    pub addressees_num: u64,
    pub estimated_fees: u64,
    pub size_of_header: u64,
    pub size_per_output: u64,
    pub size_per_input: u64,
    pub tries: u64,
}

#[derive(Debug)]
pub enum Error {
    InsufficientFunds,
}

impl<'a> BranchAndBound<'a> {
    pub fn select_coins(mut self) -> Result<Vec<&'a TxOut>, Error> {
        self.optional_utxos.sort_by(|a, b| b.value.cmp(&a.value));
        let mut selected_utxos = self.mandatory_utxos.clone();
        let sum_mandatory = self.mandatory_utxos.iter().fold(0, |sum, i| sum + i.value);
        let result = self.branch_and_bound(0, &mut selected_utxos, sum_mandatory);

        if result {
            Ok(selected_utxos)
        } else {
            // If no match, Single Random Draw
            self.single_random_draw(sum_mandatory)
        }
    }

    fn single_random_draw(mut self, sum_mandatory: u64) -> Result<Vec<&'a TxOut>, Error> {
        thread_rng().shuffle(&mut self.optional_utxos);
        let mut sum = sum_mandatory;

        let mut target = self.spending_target + self.output_cost() + self.header_cost();

        let cost_per_input = self.input_cost(1);
        let mut selected_utxos: Vec<&TxOut> = self
            .optional_utxos
            .into_iter()
            .take_while(|x| {
                sum += x.value;
                target += cost_per_input;
                sum - x.value < target
            })
            .collect();

        selected_utxos.append(&mut self.mandatory_utxos);

        if sum >= target {
            Ok(selected_utxos)
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

        if self.tries <= 0 || depth >= self.optional_utxos.len() {
            return false;
        }

        self.tries = self.tries - 1;

        // Exploring omission and inclusion branch
        let current_utxo_value = self.optional_utxos[depth].value;
        current_selection.push(self.optional_utxos[depth]);

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
