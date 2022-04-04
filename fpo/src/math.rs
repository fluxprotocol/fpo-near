use near_sdk::json_types::U128;
pub fn mean(numbers: &[u128]) -> U128 {
    let sum: u128 = numbers.iter().sum();
    U128::from(sum as u128 / numbers.len() as u128)
}
pub fn median(numbers: &mut [u128]) -> U128 {
    numbers.sort_unstable();

    let mid = numbers.len() / 2;
    if numbers.len() % 2 == 0 {
        mean(&[numbers[mid - 1], numbers[mid]]) as U128
    } else {
        U128::from(numbers[mid])
    }
}
