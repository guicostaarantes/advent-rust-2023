use std::collections::BTreeSet;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Coordinate {
    lattitude: usize,
    longitude: usize,
}

impl Coordinate {
    fn distance(&self, other: &Self) -> usize {
        self.lattitude.abs_diff(other.lattitude) + self.longitude.abs_diff(other.longitude)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Space {
    galaxies: Vec<Coordinate>,
}

impl TryFrom<(&str, usize)> for Space {
    type Error = anyhow::Error;

    fn try_from((value, exp_factor): (&str, usize)) -> Result<Self> {
        let mut galaxies = Vec::new();

        let no_of_lines = value.lines().fold(0, |acc, _| acc + 1);
        let no_of_cols = value
            .lines()
            .next()
            .context("Invalid input")?
            .chars()
            .fold(0, |acc, _| acc + 1);

        for (i, line) in value.lines().enumerate() {
            for (j, char) in line.chars().enumerate() {
                if char == '#' {
                    galaxies.push(Coordinate {
                        lattitude: i,
                        longitude: j,
                    });
                }
            }
        }

        let lines_with_galaxies = galaxies.iter().fold(BTreeSet::new(), |mut acc, v| {
            acc.insert(v.lattitude);
            acc
        });

        let cols_with_galaxies = galaxies.iter().fold(BTreeSet::new(), |mut acc, v| {
            acc.insert(v.longitude);
            acc
        });

        let mut lines_without_galaxies = Vec::new();
        for i in 0..no_of_lines {
            if !lines_with_galaxies.contains(&i) {
                lines_without_galaxies.push(i);
            }
        }

        let mut cols_without_galaxies = Vec::new();
        for i in 0..no_of_cols {
            if !cols_with_galaxies.contains(&i) {
                cols_without_galaxies.push(i);
            }
        }

        for i in lines_without_galaxies.iter().rev() {
            galaxies
                .iter_mut()
                .filter(|gal| gal.lattitude > *i)
                .for_each(|gal| {
                    gal.lattitude += exp_factor;
                });
        }

        for i in cols_without_galaxies.iter().rev() {
            galaxies
                .iter_mut()
                .filter(|gal| gal.longitude > *i)
                .for_each(|gal| {
                    gal.longitude += exp_factor;
                });
        }

        Ok(Self { galaxies })
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let space = Space::try_from((input.trim(), 1))?;

    let mut result = 0;

    for i in 0..space.galaxies.len() {
        for j in 0..i {
            result += space.galaxies[i].distance(&space.galaxies[j]);
        }
    }

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let space = Space::try_from((input.trim(), 999_999))?;

    let mut result = 0;

    for i in 0..space.galaxies.len() {
        for j in 0..i {
            result += space.galaxies[i].distance(&space.galaxies[j]);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d11::run_part_1;
    use crate::d11::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d11/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 374);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d11/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 9605127);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d11/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 82000210);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d11/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 458191688761);
    }
}
