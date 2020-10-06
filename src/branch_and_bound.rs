use bitcoin::blockdata::transaction::OutPoint;

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::convert::TryInto;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutPointValue(pub OutPoint, pub u64);

pub struct BranchAndBound {
    /// The spending target to be reached. Note that this variable should hold only the sum of the
    /// outputs, and not the fees: those will be calculated using `addresses_num`,
    /// `estimated_fee_rate`, `size_of_header`, `size_per_output`, `size_per_input`.
    pub spending_target: u64,
    /// Utxos in this vector will be always included in the result.
    /// Useful for RBF.
    pub mandatory_utxos: Vec<OutPointValue>,
    /// Utxos that could be included in the result, if mandatory_utxos
    /// are not enough.
    pub optional_utxos: Vec<OutPointValue>,
    /// How many output the transaction has. Used for estimating fees.
    pub addressees_num: u64,
    /// Estimated fee rate, in sat/vbyte
    pub estimated_fee_rate: u64,
    /// Size of the transaction header. Used for estimating fees.
    pub size_of_header: u64,
    /// Size of each transaction output. Used for estimating fees.
    pub size_per_output: u64,
    /// Size of each transaction input. Used for estimating fees.
    pub size_per_input: u64,
    /// How many times should the algorithm try to find an exact match, before answering with a
    /// suboptimal result.
    pub tries: u64,
}

#[derive(Debug)]
pub enum Error {
    InsufficientFunds,
}

impl BranchAndBound {
    /// Selects coins such that their sum is equal to or greater than the spending target, returns an
    /// Error if utxos are insufficient for reaching the spending target.
    /// It will return an exact match, if found before `self.tries` tries, otherwise a suboptimal
    /// result will be given.
    /// The suboptimal solution is obtained including all `self.mandatory_utxos`, and then
    /// picking randomly from `self.optional_utxos`, until the spending
    /// target is reached.
    pub fn select_coins(mut self) -> Result<Vec<OutPointValue>, Error> {
        self.optional_utxos.sort_by(|a, b| b.1.cmp(&a.1));
        let mut selected_utxos = self.mandatory_utxos.clone();
        let sum_mandatory = self.mandatory_utxos.iter().fold(0, |sum, i| sum + i.1);
        let result = self.branch_and_bound(0, &mut selected_utxos, sum_mandatory);

        if result {
            Ok(selected_utxos)
        } else {
            // If no match, Single Random Draw
            self.single_random_draw(sum_mandatory)
        }
    }

    fn single_random_draw(mut self, sum_mandatory: u64) -> Result<Vec<OutPointValue>, Error> {
        self.optional_utxos.shuffle(&mut thread_rng());
        let mut sum = sum_mandatory;

        let mut target = self.spending_target + self.output_cost() + self.header_cost();

        let cost_per_input = self.input_cost(1);
        // TODO: is take_while the cleanest method here?
        let mut selected_utxos: Vec<OutPointValue> = self
            .optional_utxos
            .into_iter()
            .take_while(|x| {
                sum += x.1;
                target += cost_per_input;
                sum - x.1 < target
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
        current_selection: &mut Vec<OutPointValue>,
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
        let current_utxo_value = self.optional_utxos[depth].1;
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
        self.size_per_input * self.estimated_fee_rate * input_num
    }

    fn output_cost(&self) -> u64 {
        self.addressees_num * self.size_per_output * self.estimated_fee_rate
    }

    fn header_cost(&self) -> u64 {
        self.size_of_header * self.estimated_fee_rate
    }
}
