use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Round {
    red: usize,
    green: usize,
    blue: usize,
}

impl Default for Round {
    fn default() -> Self {
        Self { red: 0, green: 0, blue: 0 }
    }
}

impl TryFrom<&str> for Round {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut result = Round::default();

        let _: Vec<_> = value
            .split(", ")
            .map(|draw| {
                let (number, color) = draw.split_once(" ").context("Bad input, no space found")?;
                match color {
                    "red" => {
                        result.red = number
                            .parse::<usize>()
                            .context("Bad input, red is not a number")?;
                    }
                    "green" => {
                        result.green = number
                            .parse::<usize>()
                            .context("Bad input, green is not a number")?;
                    }
                    "blue" => {
                        result.blue = number
                            .parse::<usize>()
                            .context("Bad input, blue is not a number")?;
                    }
                    _ => unreachable!(),
                };
                anyhow::Ok(())
            })
            .collect();

        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Game {
    number: usize,
    rounds: Vec<Round>,
}

impl TryFrom<&str> for Game {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let (number, rounds) = value
            .split_once(": ")
            .context("Bad input, no colon found")?;

        let (_, number) = number
            .split_once("Game ")
            .context("Bad input, no 'Game' found")?;

        let number = number
            .parse::<usize>()
            .context("Bad input, game is not a number")?;

        let rounds = rounds
            .split("; ")
            .map(|round| Round::try_from(round))
            .collect::<Result<Vec<Round>>>()?;

        Ok(Game { number, rounds })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Match {
    games: Vec<Game>,
}

impl TryFrom<&str> for Match {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let games = value
            .lines()
            .map(|line| Game::try_from(line))
            .collect::<Result<Vec<Game>>>()?;

        Ok(Match { games })
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let mut result = 0;

    let the_match = Match::try_from(input.trim())?;

    the_match.games.iter().for_each(|game| {
        if game
            .rounds
            .iter()
            .all(|round| round.red <= 12 && round.green <= 13 && round.blue <= 14)
        {
            result += game.number;
        }
    });

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let the_match = Match::try_from(input.trim())?;

    let result = the_match
        .games
        .iter()
        .map(|game| {
            game.rounds.iter().fold(Round::default(), |mut acc, round| {
                if round.red > acc.red {
                    acc.red = round.red;
                }
                if round.green > acc.green {
                    acc.green = round.green;
                }
                if round.blue > acc.blue {
                    acc.blue = round.blue;
                }
                acc
            })
        })
        .map(|round| round.red * round.green * round.blue)
        .sum::<usize>();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d02::run_part_1;
    use crate::d02::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d02/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 8);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d02/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 3059);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d02/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 2286);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d02/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 65371);
    }
}
