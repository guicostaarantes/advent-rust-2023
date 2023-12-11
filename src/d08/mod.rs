use std::collections::BTreeMap;

use anyhow::{Context, Result};

fn greatest_common_divisor(a: u128, b: u128) -> u128 {
    if b == 0 {
        return a;
    }
    greatest_common_divisor(b, a % b)
}

fn least_common_multiple(nums: &[u128]) -> u128 {
    if nums.len() == 1 {
        return nums[0];
    }
    let a = nums[0];
    let b = least_common_multiple(&nums[1..]);
    a * b / greatest_common_divisor(a, b)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            'L' => Ok(Self::Left),
            'R' => Ok(Self::Right),
            _ => Err(anyhow::anyhow!("Invalid step")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Position<'a> {
    name: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Path<'a> {
    leads_to: BTreeMap<Direction, Position<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Map<'a> {
    steps: Vec<Direction>,
    paths: BTreeMap<Position<'a>, Path<'a>>,
}

impl<'a> TryFrom<&'a str> for Map<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self> {
        let mut result = Self {
            steps: Vec::new(),
            paths: BTreeMap::new(),
        };

        let (steps, directions) = value.trim().split_once("\n\n").context("")?;

        result.steps = steps
            .chars()
            .map(|s| Direction::try_from(s))
            .collect::<Result<Vec<_>>>()?;

        let mut positions = Vec::new();

        let directions = directions
            .split("\n")
            .map(|dir| {
                let (name, dests) = dir.split_once(" = ").context("")?;
                let (_, dests) = dests.split_once("(").context("")?;
                let (dests, _) = dests.split_once(")").context("")?;
                let (left, right) = dests.split_once(", ").context("")?;
                positions.push(Position { name });
                Ok((name, left, right))
            })
            .collect::<Result<Vec<_>>>()?;

        for (i, dir) in directions.iter().enumerate() {
            let mut path = Path {
                leads_to: BTreeMap::new(),
            };
            path.leads_to.entry(Direction::Left).or_insert(
                positions
                    .iter()
                    .find(|po| po.name == dir.1)
                    .context("")?
                    .clone(),
            );
            path.leads_to.entry(Direction::Right).or_insert(
                positions
                    .iter()
                    .find(|po| po.name == dir.2)
                    .context("")?
                    .clone(),
            );
            result.paths.insert(positions[i].clone(), path);
        }

        Ok(result)
    }
}

#[derive(Clone)]
struct Player<'a> {
    map: &'a Map<'a>,
    current_position: Position<'a>,
    start_position: Position<'a>,
    current_step: usize,
    is_destination: &'a dyn Fn(Position<'a>) -> bool,
}

impl<'a> Player<'a> {
    fn new(
        map: &'a Map<'a>,
        start_position: Position<'a>,
        is_destination: &'a dyn Fn(Position<'a>) -> bool,
    ) -> Self {
        Self {
            map,
            current_position: start_position.clone(),
            start_position,
            current_step: 0,
            is_destination,
        }
    }
}

impl Player<'_> {
    fn walk(&mut self) {
        self.current_position = self
            .map
            .paths
            .get(&self.current_position)
            .unwrap()
            .leads_to
            .get(&self.map.steps[self.current_step % self.map.steps.len()])
            .unwrap()
            .clone();

        self.current_step += 1;
    }
}

impl Player<'_> {
    fn reset(&mut self) {
        self.current_position = self.start_position.clone();
        self.current_step = 0;
    }
}

// Todo: use ref for position
impl<'a, 'b> Player<'b> {
    fn is_at_destination(&'a self, position: Position<'b>) -> bool {
        let closure = self.is_destination;
        closure(position)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Circuit {
    steps_per_cycle: usize,
    cycles_to_start: usize,
    cycles_to_finish: usize,
    destination_indices: Vec<usize>,
}

impl<'a> Circuit {
    fn new(player: &'a mut Player<'_>) -> Self {
        let mut position_after_end_of_cycle: Vec<Position> = vec![player.current_position.clone()];
        let mut destination_indices = Vec::new();

        let steps_per_cycle = player.map.steps.len();

        let mut cycle_count = 0;
        let cycles_to_start = loop {
            for i in 0..steps_per_cycle {
                player.walk();

                if player.is_at_destination(player.current_position.clone()) {
                    destination_indices.push(cycle_count * steps_per_cycle + i);
                }
            }

            if let Some(pos) = position_after_end_of_cycle
                .iter()
                .position(|p| p.name == player.current_position.name)
            {
                break pos;
            } else {
                position_after_end_of_cycle.push(player.current_position.clone());
            }

            cycle_count += 1;
        };

        let cycles_to_finish = position_after_end_of_cycle.len() - cycles_to_start;

        let steps_until_circuit = cycles_to_start * steps_per_cycle;

        let destination_indices = destination_indices
            .iter()
            .filter(|d| **d >= steps_until_circuit)
            .map(|d| *d - steps_until_circuit + 1)
            .collect::<Vec<_>>();

        player.reset();

        Self {
            steps_per_cycle,
            cycles_to_start,
            cycles_to_finish,
            destination_indices,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MultipleCircuits {
    circuits: Vec<Circuit>,
}

impl MultipleCircuits {
    fn calculate_min_steps_to_destination(&self) -> Result<u128> {
        let assumption = self.circuits.iter().all(|c| {
            c.steps_per_cycle == self.circuits[0].steps_per_cycle
                && c.cycles_to_start == self.circuits[0].cycles_to_start
        });

        if !assumption {
            return Err(anyhow::anyhow!(
                "Can not guarantee that this method will yield the correct result"
            ));
        }

        /*
         * When a circuit is found and there is at least one destination inside it, it means there will be
         * infinite destinations, which can be found by:
         *
         * let mut solutions = Vec::new();
         * let mut i = 0;
         * loop {
         *     for j in c.destinations_inside_circuit.iter() {
         *         solutions.push(i * (c.cycles_to_finish * c.steps_per_cycle) + (c.cycles_to_start * c.steps_per_cycle) + j);
         *     }
         *     i += 1;
         * }
         *
         *
         **/

        let values = self
            .circuits
            .iter()
            .map(|c| (c.steps_per_cycle * c.cycles_to_start + c.destination_indices[0]) as u128)
            .collect::<Vec<u128>>();

        Ok(least_common_multiple(&values))
    }
}

pub fn run_part_1(input: String) -> Result<u128> {
    let map = Map::try_from(input.trim())?;

    let is_destination = |pos: Position| pos.name == "ZZZ";

    let mut players = map
        .paths
        .keys()
        .filter(|pos| pos.name == "AAA")
        .map(|sp| Player::new(&map, sp.clone(), &is_destination))
        .collect::<Vec<_>>();

    let circuits = players
        .iter_mut()
        .map(|pl| Circuit::new(pl))
        .collect::<Vec<_>>();

    let max_possible_value_before_solving_with_circuits = circuits
        .iter()
        .map(|c| c.steps_per_cycle * (c.cycles_to_start + c.cycles_to_finish))
        .max()
        .unwrap();

    for k in 1..=max_possible_value_before_solving_with_circuits {
        players.iter_mut().for_each(|pl| pl.walk());

        if players
            .iter()
            .all(|pl| pl.is_at_destination(pl.current_position.clone()))
        {
            let k = k as u128;
            return Ok(k);
        }
    }

    let multiple_circuits = MultipleCircuits { circuits };

    multiple_circuits.calculate_min_steps_to_destination()
}

pub fn run_part_2(input: String) -> Result<u128> {
    let map = Map::try_from(input.trim())?;

    let is_destination = |pos: Position| pos.name.chars().last() == Some('Z');

    let mut players = map
        .paths
        .keys()
        .filter(|pos| pos.name.chars().last() == Some('A'))
        .map(|sp| Player::new(&map, sp.clone(), &is_destination))
        .collect::<Vec<_>>();

    let circuits = players
        .iter_mut()
        .map(|pl| Circuit::new(pl))
        .collect::<Vec<_>>();

    let max_possible_value_before_circuits = circuits
        .iter()
        .map(|c| c.steps_per_cycle * (c.cycles_to_start + c.cycles_to_finish))
        .max()
        .unwrap();

    for k in 1..=max_possible_value_before_circuits {
        players.iter_mut().for_each(|pl| pl.walk());

        if players
            .iter()
            .all(|pl| pl.is_at_destination(pl.current_position.clone()))
        {
            let k = k as u128;
            return Ok(k);
        }
    }

    let multiple_circuits = MultipleCircuits { circuits };

    multiple_circuits.calculate_min_steps_to_destination()
}

#[cfg(test)]
mod tests {
    use crate::d08::run_part_1;
    use crate::d08::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d08/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 2);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d08/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 22411);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d08/test2.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 6);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d08/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 11188774513823);
    }
}
