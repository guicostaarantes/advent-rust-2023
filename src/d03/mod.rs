use std::collections::HashMap;

use anyhow::{Context, Result};

#[derive(PartialEq, Eq, Hash, Debug)]
struct Position {
    ln: usize,
    col: usize,
}

#[derive(Debug)]
struct PartNumber {
    value: String,
    start_position: Position,
}

#[derive(Debug)]
struct Schematic {
    value: Vec<Vec<char>>,
}

impl Schematic {
    fn check_surroundings(&self, pn: &PartNumber) -> HashMap<Position, &char> {
        let mut result = HashMap::new();
        let mut surroundings = vec![
            (0usize, 0usize),
            (1, 0),
            (2, 0),
            (0, pn.value.len() + 1),
            (1, pn.value.len() + 1),
            (2, pn.value.len() + 1),
        ];
        for k in 1..=pn.value.len() {
            surroundings.push((0, k));
            surroundings.push((2, k));
        }
        let _ = surroundings
            .iter()
            .map(|su| {
                let ln = (su.0 + pn.start_position.ln)
                    .checked_sub(1)
                    .context("First line")?;
                let col = (su.1 + pn.start_position.col)
                    .checked_sub(1)
                    .context("First column")?;
                let char_at_position = self
                    .value
                    .get(ln)
                    .context("Last line")?
                    .get(col)
                    .context("Last column")?;
                result.insert(Position { ln, col }, char_at_position);
                anyhow::Ok(())
            })
            .collect::<Vec<Result<()>>>();
        result
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let mut result = 0;

    let schematic = Schematic {
        value: input
            .trim()
            .lines()
            .map(|line| line.chars().collect())
            .collect(),
    };

    let mut current_part_number: Option<PartNumber> = None;

    for (i, line) in schematic.value.iter().enumerate() {
        for (j, char) in line.iter().enumerate() {
            if char.is_digit(10) {
                if let Some(ref mut pn) = current_part_number {
                    // continue capturning part number
                    pn.value.push(*char);
                } else {
                    // start capturing part number
                    current_part_number = Some(PartNumber {
                        value: String::from(*char),
                        start_position: Position { ln: i, col: j },
                    });
                }
            } else {
                if let Some(ref mut pn) = current_part_number {
                    // finished capturing part number
                    let surroundings = schematic.check_surroundings(pn);
                    if surroundings
                        .values()
                        .any(|su| **su != '.' && !su.is_digit(10))
                    {
                        result += pn.value.parse::<usize>().context("Not a number")?;
                    }
                    current_part_number = None;
                }
            }
        }
    }

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let mut gear_map: HashMap<Position, Vec<usize>> = HashMap::new();

    let schematic = Schematic {
        value: input
            .trim()
            .lines()
            .map(|line| line.chars().collect())
            .collect(),
    };

    let mut current_part_number: Option<PartNumber> = None;

    for (i, line) in schematic.value.iter().enumerate() {
        for (j, char) in line.iter().enumerate() {
            if char.is_digit(10) {
                if let Some(ref mut pn) = current_part_number {
                    // continue capturning part number
                    pn.value.push(*char);
                } else {
                    // start capturing part number
                    current_part_number = Some(PartNumber {
                        value: String::from(*char),
                        start_position: Position { ln: i, col: j },
                    });
                }
            } else {
                if let Some(ref mut pn) = current_part_number {
                    // finished capturing part number
                    let surroundings = schematic.check_surroundings(pn);
                    for su in surroundings {
                        if su.1 == &'*' {
                            gear_map
                                .entry(su.0)
                                .and_modify(|ve| ve.push(pn.value.parse::<usize>().unwrap()))
                                .or_insert(vec![pn.value.parse::<usize>().unwrap()]);
                        }
                    }
                    current_part_number = None;
                }
            }
        }
    }

    let result = gear_map.values().filter(|v| v.len() == 2).map(|v| v[0] * v[1]).sum();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d03::run_part_1;
    use crate::d03::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d03/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 4361);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d03/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 537732);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d03/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 467835);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d03/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 84883664);
    }
}
