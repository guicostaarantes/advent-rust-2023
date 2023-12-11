use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Race {
    time: usize,
    distance_to_beat: usize,
}

// for part 2
impl TryFrom<&str> for Race {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let (time, distance_to_beat) = value.split_once("\n").context("Bad input")?;
        let (_, time) = time.split_once(":").context("Bad input")?;
        let (_, distance_to_beat) = distance_to_beat.split_once(":").context("Bad input")?;
        let time = time
            .split(" ")
            .filter(|t| !t.is_empty())
            .collect::<Vec<&str>>()
            .join("")
            .parse::<usize>()?;
        let distance_to_beat = distance_to_beat
            .split(" ")
            .filter(|t| !t.is_empty())
            .collect::<Vec<&str>>()
            .join("")
            .parse::<usize>()?;

        Ok(Self { time, distance_to_beat })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RaceBoard {
    races: Vec<Race>,
}

impl TryFrom<&str> for RaceBoard {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut races = Vec::new();

        let (times, distances) = value.split_once("\n").context("Bad input")?;
        let (_, times) = times.split_once(":").context("Bad input")?;
        let (_, distances) = distances.split_once(":").context("Bad input")?;
        let times = times.split(" ").filter(|t| !t.is_empty());
        let mut distances = distances.split(" ").filter(|t| !t.is_empty());

        for time in times {
            let time = time.parse::<usize>().context("Bad input")?;
            let distance_to_beat = distances
                .next()
                .context("Bad input")?
                .parse::<usize>()
                .context("Bad input")?;

            races.push(Race {
                time,
                distance_to_beat,
            });
        }

        Ok(Self { races })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Boat {
    speed: usize,
}

impl Boat {
    fn calculate_distance(&self, race_time: usize, charge_time: usize) -> usize {
        (charge_time * self.speed) * (race_time - charge_time)
    }

    fn will_win(&self, race: &Race, charge_time: usize) -> bool {
        self.calculate_distance(race.time, charge_time) > race.distance_to_beat
    }

    fn how_many_diff_ways_to_win(&self, race: &Race) -> usize {
        let mut left_margin = 0;
        let mut right_margin = race.time;
        let min_to_win = loop {
            let guess = (left_margin + right_margin) / 2;
            if guess == left_margin {
                if self.will_win(&race, left_margin) {
                    break left_margin;
                } else {
                    break right_margin;
                }
            }
            if self.will_win(&race, guess) {
                right_margin = guess;
            } else {
                left_margin = guess;
            }
        };

        let mut left_margin = 0;
        let mut right_margin = race.time;
        let max_to_win = loop {
            let guess = (left_margin + right_margin) / 2;
            if guess == left_margin {
                if self.will_win(&race, right_margin) {
                    break right_margin;
                } else {
                    break left_margin;
                }
            }
            if self.will_win(&race, guess) {
                left_margin = guess;
            } else {
                right_margin = guess;
            }
        };

        max_to_win - min_to_win + 1
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let boat = Boat { speed: 1 };

    let race_board = RaceBoard::try_from(input.trim())?;

    let mut result = 1;

    for race in race_board.races.iter() {
        result *= boat.how_many_diff_ways_to_win(&race);
    }

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let boat = Boat { speed: 1 };

    let race = Race::try_from(input.trim())?;

    Ok(boat.how_many_diff_ways_to_win(&race))
}

#[cfg(test)]
mod tests {
    use crate::d06::run_part_1;
    use crate::d06::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d06/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 288);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d06/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 1731600);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d06/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 71503);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d06/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 40087680);
    }
}
