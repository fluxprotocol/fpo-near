use near_sdk::{env, StorageUsage, Balance, AccountId, Promise};
use near_sdk::json_types::U128;

pub fn refund_storage(initial_storage: StorageUsage, sender_id: AccountId) {
    let current_storage = env::storage_usage();
    let attached_deposit = env::attached_deposit();
    let refund_amount = if current_storage > initial_storage {
        let required_deposit =
            Balance::from(current_storage - initial_storage) * env::storage_byte_cost();
        assert!(
            required_deposit <= attached_deposit,
            "The required attached deposit is {}, but the given attached deposit is is {}",
            required_deposit,
            attached_deposit,
        );
        attached_deposit - required_deposit
    } else {
        attached_deposit + Balance::from(initial_storage - current_storage) * env::storage_byte_cost()
    };
    if refund_amount > 0 {
        Promise::new(sender_id).transfer(refund_amount);
    }
}

pub fn mean(numbers: &Vec<u128>) -> U128 {
    let sum: u128 = numbers.iter().sum();
    U128::from( sum as u128 / numbers.len() as u128)

}
pub fn median(numbers: &mut Vec<u128>) -> U128 {
    numbers.sort();

    let mid = numbers.len() / 2;
    if numbers.len() % 2 == 0 {
        mean(&vec![numbers[mid - 1], numbers[mid]]) as U128
    } else {
        U128::from(numbers[mid])
    }
}
