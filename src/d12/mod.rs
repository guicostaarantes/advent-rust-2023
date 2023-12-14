use std::collections::BTreeMap;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Spring {
    Operational,
    Damaged,
    Unknown,
}

impl TryFrom<char> for Spring {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            '.' => Ok(Self::Operational),
            '#' => Ok(Self::Damaged),
            '?' => Ok(Self::Unknown),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Arrangement {
    springs: Vec<Spring>,
    sequences: Vec<usize>,
    cache: BTreeMap<(usize, usize), usize>,
}

impl TryFrom<&str> for Arrangement {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let (springs, sequences) = value.split_once(" ").context("Space not found")?;
        let springs = springs
            .chars()
            .map(|s| Spring::try_from(s))
            .collect::<Result<Vec<Spring>>>()?;
        let sequences = sequences
            .split(",")
            .map(|s| s.parse::<usize>().context("Not a valid number"))
            .collect::<Result<Vec<usize>>>()?;
        let cache = BTreeMap::new();

        Ok(Self {
            springs,
            sequences,
            cache,
        })
    }
}

impl Arrangement {
    fn find_possible_solutions(
        &mut self,
        starting_from_spring: usize,
        starting_from_sequence: usize,
    ) -> usize {
        // at some points the recursion will get back to a state that was already calculated, so a
        // cache is helpful here
        if let Some(cache_hit) = self
            .cache
            .get(&(starting_from_spring, starting_from_sequence))
        {
            return *cache_hit;
        }

        // end of recursion: at some point, one of these trivial cases will happen:
        // - a sequence remains to be evaluated but it ran out of springs: not a solution
        // - a damaged spring is still awaiting a sequence to match but it ran out of sequences: not a solution
        // - it ran out of sequences to be evaluated and there are no damaged springs remaining: A SOLUTION
        if starting_from_sequence >= self.sequences.len() {
            if starting_from_spring >= self.springs.len()
                || !self.springs[starting_from_spring..].contains(&Spring::Damaged)
            {
                return 1;
            } else {
                return 0;
            };
        }
        if starting_from_spring >= self.springs.len() {
            return 0;
        }

        // running the function recursively depending on what is the spring at the starting_from_spring position
        let next_run = match self.springs[starting_from_spring] {
            Spring::Operational => {
                // moves to the next spring, remains in the same sequence
                self.find_possible_solutions(starting_from_spring + 1, starting_from_sequence)
            }
            Spring::Damaged => {
                // this can be the start of a damaged sequence
                let current_sequence = self.sequences[starting_from_sequence];
                if starting_from_spring + current_sequence > self.springs.len()
                    || self.springs[starting_from_spring..starting_from_spring + current_sequence]
                        .contains(&Spring::Operational)
                    || self.springs.get(starting_from_spring + current_sequence)
                        == Some(&Spring::Damaged)
                {
                    // evaluates the next n springs that can be a damaged sequence
                    // if there are any operational springs in this interval, or if the spring
                    // right after the end is a damaged one, this can't be the start of the
                    // sequence with that length
                    0
                } else {
                    // moves to the next n springs and to the next sequence
                    self.find_possible_solutions(
                        starting_from_spring + current_sequence + 1,
                        starting_from_sequence + 1,
                    )
                }
            }
            Spring::Unknown => {
                // this can be the start of a damaged sequence or an operational spring
                let current_sequence = self.sequences[starting_from_sequence];
                if starting_from_spring + current_sequence > self.springs.len()
                    || self.springs[starting_from_spring..starting_from_spring + current_sequence]
                        .contains(&Spring::Operational)
                    || self.springs.get(starting_from_spring + current_sequence)
                        == Some(&Spring::Damaged)
                {
                    // evaluates the next n springs that can be a damaged sequence
                    // if there are any operational springs in this interval, or if the spring
                    // right after the end is a damaged one, the current spring can only be
                    // an operational one
                    self.find_possible_solutions(starting_from_spring + 1, starting_from_sequence)
                } else {
                    // else it can be both, so it calculates the possible solutions if this is an
                    // operational spring plus if this is a damaged spring
                    self.find_possible_solutions(
                        starting_from_spring + current_sequence + 1,
                        starting_from_sequence + 1,
                    ) + self
                        .find_possible_solutions(starting_from_spring + 1, starting_from_sequence)
                }
            }
        };

        self.cache
            .insert((starting_from_spring, starting_from_sequence), next_run);
        return next_run;
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let mut arrangements = input
        .trim()
        .lines()
        .map(|arr| Arrangement::try_from(arr))
        .collect::<Result<Vec<Arrangement>>>()?;

    let result = arrangements
        .iter_mut()
        .map(|arr| arr.find_possible_solutions(0, 0))
        .sum();

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let mut arrangements = input
        .trim()
        .lines()
        .map(|line| -> Result<String, _> {
            let (springs, sequences) = line.split_once(" ").context("Space not found")?;
            let five_springs = (0..5).map(|_| springs).collect::<Vec<&str>>().join("?");
            let five_sequences = (0..5).map(|_| sequences).collect::<Vec<&str>>().join(",");
            Ok(format!("{} {}", five_springs, five_sequences))
        })
        .collect::<Result<Vec<String>>>()?
        .iter()
        .map(|arr| Arrangement::try_from(&arr[..]))
        .collect::<Result<Vec<Arrangement>>>()?;

    let result = arrangements
        .iter_mut()
        .map(|arr| arr.find_possible_solutions(0, 0))
        .sum();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d12::run_part_1;
    use crate::d12::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d12/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 21);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d12/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 7857);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d12/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 525152);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d12/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 28606137449920);
    }
}
