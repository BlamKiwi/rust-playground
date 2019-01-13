use std::slice;

#[derive(Debug)]
pub enum CombinatorialError {
    OutputSizeTooLarge,
}

fn add_with_carry(indexes: &mut [usize], base: usize) -> (usize, usize) {
    for (i, index) in indexes.iter_mut().enumerate() {
        *index = (*index + 1) % base;

        if *index != 0 {
            return (i, *index);
        }
    }

    (0, 0)
}

fn swizzle<'a, T>(source: &'a [T], indexes: &[usize]) -> Vec<&'a T> {
    let mut result = Vec::new();

    for &swizzle in indexes {
        result.push(&source[swizzle]);
    }

    result
}

/// Generates General Permutations from a given slice. (Unordered with Duplicates)
pub struct GeneralPermutationsIterator<'a, T> {
    source: &'a [T],
    indexes: std::vec::Vec<usize>,
    modified: bool,
}

impl<'a, T> GeneralPermutationsIterator<'a, T> {
    #[warn(clippy::new_ret_no_self)]
    pub fn new(
        source: &'a [T],
        output_size: usize,
    ) -> Result<GeneralPermutationsIterator<'a, T>, CombinatorialError> {
        if source.len() == 0 && output_size > 0 {
            return Err(CombinatorialError::OutputSizeTooLarge);
        }

        Ok(GeneralPermutationsIterator {
            source,
            indexes: vec![0; output_size],
            modified: false,
        })
    }
}

impl<'a, T> std::iter::Iterator for GeneralPermutationsIterator<'a, T> {
    type Item = std::vec::Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.indexes.iter().all(|&x| x == 0) && self.modified {
            None
        } else {
            let result = swizzle(self.source, &self.indexes);

            add_with_carry(&mut self.indexes, self.source.len());
            self.modified = true;

            Some(result)
        }
    }
}

/// Generates Simple Combinations from a given slice. (Ordered without Duplicates)
pub struct SimpleCombinationsIterator<'a, T> {
    source: &'a [T],
    skip: std::vec::Vec<usize>,
    consumed: bool,
}

impl<'a, T> SimpleCombinationsIterator<'a, T> {
    #[warn(clippy::new_ret_no_self)]
    pub fn new(
        source: &'a [T],
        output_size: usize,
    ) -> Result<SimpleCombinationsIterator<'a, T>, CombinatorialError> {
        if output_size > source.len() {
            return Err(CombinatorialError::OutputSizeTooLarge);
        }

        let delta = source.len() - output_size;
        let mut skip = Vec::new();
        for i in 0..delta {
            skip.push(delta - i - 1);
        }

        Ok(SimpleCombinationsIterator {
            source,
            skip,
            consumed: false,
        })
    }

    fn increment_skip_hole(&mut self) -> Option<(usize, usize)> {
        for (i, index) in self.skip.iter_mut().enumerate() {
            if *index == self.source.len() - 1 - i {
                continue;
            } else {
                *index += 1;
                return Some((i, *index));
            }
        }
        None
    }

    fn generate_result(&self) -> std::vec::Vec<&'a T> {
        let mut skip_iter = self.skip.iter().rev().peekable();
        self.source
            .iter()
            .enumerate()
            .filter(|(i, _)| match skip_iter.peek() {
                Some(&&x) if x == *i => {
                    skip_iter.next();
                    false
                }
                _ => true,
            })
            .map(|(_, item)| item)
            .collect()
    }
}

impl<'a, T> std::iter::Iterator for SimpleCombinationsIterator<'a, T> {
    type Item = std::vec::Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.consumed {
            None
        } else {
            let result = self.generate_result();

            match self.increment_skip_hole() {
                Some((i, skip)) => {
                    for (j, index) in self.skip.iter_mut().take(i).enumerate() {
                        *index = skip + (i - j);
                    }
                }
                None => self.consumed = true,
            }

            Some(result)
        }
    }
}

/// Generates Simple Permutations from a given slice. (Unordered without Duplicates)
pub struct SimplePermutationsIterator<'a, T> {
    source: &'a [T],
    seed_iter: SimpleCombinationsIterator<'a, usize>,
    _seed: Vec<usize>,
    indexes: Vec<usize>,
    counters: Vec<usize>,
    current_output: usize,
}

impl<'a, T> SimplePermutationsIterator<'a, T> {
    #[warn(clippy::new_ret_no_self)]
    pub fn new(
        source: &'a [T],
        output_size: usize,
    ) -> Result<SimplePermutationsIterator<'a, T>, CombinatorialError> {
        if output_size > source.len() {
            return Err(CombinatorialError::OutputSizeTooLarge);
        }
        // REQUIRED TO ALLOW ITERATOR TO POINT INTO SEED VECTOR
        // LIFETIME IS EQUIVALENT TO STRUCT LIFETIME
        let seed: Vec<usize> = (0..source.len()).collect();
        let seed_iter = SimpleCombinationsIterator::new(
            unsafe { slice::from_raw_parts(seed.as_ptr(), source.len()) },
            output_size,
        )
        .unwrap();

        Ok(SimplePermutationsIterator {
            source,
            _seed: seed,
            seed_iter,
            indexes: vec![0; output_size],
            counters: vec![0; output_size],
            current_output: output_size,
        })
    }

    fn try_gen_permutation(&mut self) -> Option<std::vec::Vec<&'a T>> {
        if self.counters[self.current_output] < self.current_output {
            // Swizzle indexes to transition permutation
            let j = (self.current_output % 2) * self.counters[self.current_output];
            self.indexes.swap(j, self.current_output);

            // Consume permutation
            self.counters[self.current_output] += 1;
            self.current_output = 1;

            Some(swizzle(self.source, &self.indexes))
        } else {
            // Reset counter for swizzle position and consider next entry
            self.counters[self.current_output] = 0;
            self.current_output += 1;

            None
        }
    }

    fn try_next_seed(&mut self) -> Option<std::vec::Vec<&'a T>> {
        match self.seed_iter.next() {
            Some(seed_indexes) => {
                // Reset seed and permutation state
                let output_size = self.counters.len();
                self.indexes.clear();
                self.indexes.extend(seed_indexes.iter().map(|&&x| x));
                self.counters.clear();
                self.counters.resize(output_size, 0);
                self.current_output = 1;

                // Generate the identity permutation
                Some(swizzle(self.source, &self.indexes))
            }
            None => None,
        }
    }
}

impl<'a, T> std::iter::Iterator for SimplePermutationsIterator<'a, T> {
    type Item = std::vec::Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_output < self.counters.len() {
            if let Some(result) = self.try_gen_permutation() {
                return Some(result);
            }
        }
        self.try_next_seed()
    }
}

/// Generates General Combinations from a given slice. (Ordered with Duplicates)
pub struct GeneralCombinationsIterator<'a, T> {
    source: &'a [T],
    indexes: std::vec::Vec<usize>,
    modified: bool,
}

impl<'a, T> GeneralCombinationsIterator<'a, T> {
    #[warn(clippy::new_ret_no_self)]
    pub fn new(
        source: &'a [T],
        output_size: usize,
    ) -> Result<GeneralCombinationsIterator<'a, T>, CombinatorialError> {
        if source.len() == 0 && output_size > 0 {
            return Err(CombinatorialError::OutputSizeTooLarge);
        }

        Ok(GeneralCombinationsIterator {
            source,
            indexes: vec![0; output_size],
            modified: false,
        })
    }
}

impl<'a, T> std::iter::Iterator for GeneralCombinationsIterator<'a, T> {
    type Item = std::vec::Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.indexes.iter().all(|&x| x == 0) && self.modified {
            None
        } else {
            let result = swizzle(self.source, &self.indexes);

            let (modified_index, new_index_value) =
                add_with_carry(&mut self.indexes, self.source.len());
            for index in self.indexes.iter_mut().take(modified_index) {
                *index = new_index_value;
            }
            self.modified = true;

            Some(result)
        }
    }
}

/// Generates all Subsets from a given slice.
pub struct SubsetIterator<'a, T> {
    source: &'a [T],
    combinations_iter: SimpleCombinationsIterator<'a, T>,
    sizes: std::ops::RangeInclusive<usize>,
}

impl<'a, T> std::iter::Iterator for SubsetIterator<'a, T> {
    type Item = std::vec::Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.combinations_iter.next() {
            Some(x) => Some(x),
            None => match self.sizes.next() {
                Some(s) => {
                    self.combinations_iter =
                        SimpleCombinationsIterator::new(self.source, s).unwrap();
                    self.combinations_iter.next()
                }
                None => None,
            },
        }
    }
}

impl<'a, T> SubsetIterator<'a, T> {
    #[warn(clippy::new_ret_no_self)]
    pub fn new(source: &'a [T]) -> SubsetIterator<'a, T> {
        SubsetIterator {
            source,
            combinations_iter: SimpleCombinationsIterator::new(source, 0).unwrap(),
            sizes: 1..=source.len(),
        }
    }
}

/// Generates all Sublists from a given slice.
pub struct SublistIterator<'a, T> {
    source: &'a [T],
    combinations_iter: SimplePermutationsIterator<'a, T>,
    sizes: std::ops::RangeInclusive<usize>,
}

impl<'a, T> std::iter::Iterator for SublistIterator<'a, T> {
    type Item = std::vec::Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.combinations_iter.next() {
            Some(x) => Some(x),
            None => match self.sizes.next() {
                Some(s) => {
                    self.combinations_iter =
                        SimplePermutationsIterator::new(self.source, s).unwrap();
                    self.combinations_iter.next()
                }
                None => None,
            },
        }
    }
}

impl<'a, T> SublistIterator<'a, T> {
    #[warn(clippy::new_ret_no_self)]
    pub fn new(source: &'a [T]) -> SublistIterator<'a, T> {
        SublistIterator {
            source,
            combinations_iter: SimplePermutationsIterator::new(source, 0).unwrap(),
            sizes: 1..=source.len(),
        }
    }
}
