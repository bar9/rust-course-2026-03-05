fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];

    // Keep only even numbers
    let evens: Vec<_> = numbers.iter().filter(|&x| x % 2 == 0).cloned().collect(); // [2, 4, 6]
    println!("{:?}", evens);
}
