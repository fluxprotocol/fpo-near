use near_sdk::json_types::U128;
pub fn mean(numbers: &Vec<u128>) -> U128 {
    let sum: u128 = numbers.iter().sum();
    U128::from(sum as u128 / numbers.len() as u128)
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
