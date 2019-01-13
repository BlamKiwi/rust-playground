extern crate combinatorial;

fn main() {
    {
        let nums = &[1, 2, 3, 4, 5, 6, 7, 8];
        let numbers = combinatorial::SubsetIterator::new(nums);

        for (i, perm) in numbers.enumerate() {
            println!("{}, {:?}", i + 1, perm);
        }
    }
}
