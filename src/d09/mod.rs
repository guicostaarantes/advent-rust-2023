use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Sequence {
    levels: Vec<Vec<isize>>,
}

impl TryFrom<&str> for Sequence {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let original = value
            .split(" ")
            .map(|v| v.parse::<isize>().context("Error in parse"))
            .collect::<Result<Vec<isize>>>()?;

        Ok(Self {
            levels: vec![original],
        })
    }
}

impl Sequence {
    fn calculate_next_level_in_loop(&mut self) -> Result<()> {
        let last_values = self.levels.iter().last().context("Empty list")?;

        if last_values.iter().all(|v| *v == 0) {
            return Ok(());
        }

        let mut to_add = Vec::new();
        for k in 0..last_values.len() - 1 {
            to_add.push(last_values[k + 1] - last_values[k]);
        }

        self.levels.push(to_add);

        self.calculate_next_level_in_loop()
    }

    fn prev_value(&self) -> Result<isize> {
        let sum = self
            .levels
            .iter()
            .enumerate()
            .map(|(i, seq)| {
                let multiply_by = if i % 2 == 0 { 1 } else { -1 };
                let val = seq.iter().next().context("Empty list")?;
                Ok(*val * multiply_by)
            })
            .collect::<Result<Vec<isize>>>()?
            .iter()
            .sum();

        Ok(sum)
    }

    fn next_value(&self) -> Result<isize> {
        let sum = self
            .levels
            .iter()
            .map(|seq| {
                let val = seq.iter().last().context("Empty list")?;
                Ok(*val)
            })
            .collect::<Result<Vec<isize>>>()?
            .iter()
            .sum();

        Ok(sum)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Sequences {
    values: Vec<Sequence>,
}

impl TryFrom<&str> for Sequences {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let values = value
            .split("\n")
            .map(|seq| Sequence::try_from(seq))
            .collect::<Result<Vec<Sequence>>>()?;

        Ok(Self { values })
    }
}

pub fn run_part_1(input: String) -> Result<isize> {
    let mut sequences = Sequences::try_from(input.trim())?;

    let next_values = sequences
        .values
        .iter_mut()
        .map(|seq| {
            seq.calculate_next_level_in_loop()?;
            let next_value = seq.next_value()?;
            Ok(next_value)
        })
        .collect::<Result<Vec<isize>>>()?;

    Ok(next_values.iter().sum())
}

pub fn run_part_2(input: String) -> Result<isize> {
    let mut sequences = Sequences::try_from(input.trim())?;

    let prev_values = sequences
        .values
        .iter_mut()
        .map(|seq| {
            seq.calculate_next_level_in_loop()?;
            let prev_value = seq.prev_value()?;
            Ok(prev_value)
        })
        .collect::<Result<Vec<isize>>>()?;

    Ok(prev_values.iter().sum())
}

#[cfg(test)]
mod tests {
    use crate::d09::run_part_1;
    use crate::d09::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d09/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 114);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d09/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 1581679977);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d09/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 2);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d09/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 889);
    }
}
