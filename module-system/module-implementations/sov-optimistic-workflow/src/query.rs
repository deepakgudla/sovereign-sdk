use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sov_rollup_interface::zk::{ValidityCondition, ValidityConditionChecker, Zkvm};
use sov_state::WorkingSet;

use super::AttesterIncentives;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Response {
    pub value: u64,
}

impl<
        C: sov_modules_api::Context,
        Vm: Zkvm,
        Cond: ValidityCondition,
        Checker: ValidityConditionChecker<Cond> + BorshDeserialize + BorshSerialize,
    > AttesterIncentives<C, Vm, Cond, Checker>
{
    /// Queries the state of the module.
    pub fn get_bond_amount(
        &self,
        address: C::Address,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Response {
        Response {
            value: self
                .bonded_attesters
                .get(&address, working_set)
                .unwrap_or_default(), // self.value.get(working_set),
        }
    }
}
