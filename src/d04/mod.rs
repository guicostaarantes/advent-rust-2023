use anyhow::{Context, Result};

struct Card {
    winning_numbers: Vec<usize>,
    your_numbers: Vec<usize>,
}

impl TryFrom<&str> for Card {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let value = value
            .split(" ")
            .filter(|w| !w.is_empty())
            .collect::<Vec<&str>>()
            .join(" ");

        let (_, data) = value
            .split_once(": ")
            .context("Bad input, colon is missing")?;

        let (win, your) = data
            .split_once(" | ")
            .context("Bad input, pipe is missing")?;
        let win = win
            .split(" ")
            .map(|n| n.parse::<usize>().context("Bad input, not a number"))
            .collect::<Result<Vec<usize>>>()
            .context("Bad input at winning numbers")?;
        let your = your
            .split(" ")
            .map(|n| n.parse::<usize>().context("Bad input, not a number"))
            .collect::<Result<Vec<usize>>>()
            .context("Bad input at your numbers")?;

        Ok(Card {
            winning_numbers: win,
            your_numbers: your,
        })
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let cards = input
        .trim()
        .lines()
        .map(|line| Card::try_from(line))
        .collect::<Result<Vec<Card>>>()?;

    let result = cards
        .iter()
        .map(|card| {
            let mut result = 0;
            card.your_numbers.iter().for_each(|your| {
                if card.winning_numbers.contains(your) {
                    if result == 0 {
                        result = 1;
                    } else {
                        result *= 2;
                    }
                }
            });
            result
        })
        .sum();

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let mut cards = Vec::new();
    let mut copies = Vec::new();

    let _ = input
        .trim()
        .lines()
        .map(|line| {
            let card = Card::try_from(line)?;
            copies.push(1);
            cards.push(card);
            Ok(())
        })
        .collect::<Result<Vec<()>>>();

    for k in 0..cards.len() {
        let card = &cards[k];
        let mut points = 0;
        card.your_numbers.iter().for_each(|your| {
            if card.winning_numbers.contains(your) {
                points += 1;
            }
        });
        for u in 0..points {
            let raise = copies[k];
            if let Some(copy) = copies.get_mut(k + u + 1) {
                *copy += raise;
            }
        }
    }

    let result = copies.iter().sum();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d04::run_part_1;
    use crate::d04::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d04/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 13);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d04/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 28750);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d04/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 30);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d04/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 10212704);
    }
}
