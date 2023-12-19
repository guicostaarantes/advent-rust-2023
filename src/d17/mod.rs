use std::collections::BinaryHeap;
use std::collections::HashMap;

use anyhow::{Context, Result};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Coordinate {
    x: usize,
    y: usize,
}

impl std::fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Co({},{})", self.x, self.y)
    }
}

impl Coordinate {
    fn single_step(&self, dir: &Direction) -> Self {
        let mut x = self.x;
        let mut y = self.y;

        match dir {
            Direction::North => x = x.checked_sub(1).unwrap_or(usize::MAX),
            Direction::West => y = y.checked_sub(1).unwrap_or(usize::MAX),
            Direction::South => x = x.checked_add(1).unwrap_or(usize::MAX),
            Direction::East => y = y.checked_add(1).unwrap_or(usize::MAX),
        };

        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::West => Direction::East,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Path {
    coordinate: Coordinate,
    going_towards: Direction,
    going_towards_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PathWithCost {
    total_cost: usize,
    coordinate: Coordinate,
    going_towards: Direction,
    going_towards_count: usize,
}

impl Ord for PathWithCost {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.total_cost.cmp(&other.total_cost).reverse()
    }
}

impl PartialOrd for PathWithCost {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.total_cost.partial_cmp(&other.total_cost) {
            Some(cmp) => Some(cmp.reverse()),
            None => None,
        }
    }
}

impl From<&PathWithCost> for Path {
    fn from(value: &PathWithCost) -> Self {
        Self {
            coordinate: value.coordinate.clone(),
            going_towards: value.going_towards.clone(),
            going_towards_count: value.going_towards_count,
        }
    }
}

impl Path {
    fn add_cost(self, cost: usize) -> PathWithCost {
        PathWithCost {
            total_cost: cost,
            coordinate: self.coordinate,
            going_towards: self.going_towards,
            going_towards_count: self.going_towards_count,
        }
    }
}

#[derive(Clone)]
struct Map {
    nodes: HashMap<Coordinate, usize>,
    min_steps: Option<usize>,
    max_steps: Option<usize>,
    unexplored_paths: BinaryHeap<PathWithCost>,
    explored_paths: HashMap<Path, usize>,
}

impl TryFrom<&str> for Map {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut nodes = HashMap::new();

        value
            .lines()
            .enumerate()
            .map(|(i, line)| {
                line.chars()
                    .enumerate()
                    .map(|(j, char)| {
                        nodes.insert(
                            Coordinate { x: i, y: j },
                            char.to_digit(10).context("Invalid input: not a digit")? as usize,
                        );
                        anyhow::Ok(())
                    })
                    .count();
            })
            .count();

        Ok(Self {
            nodes,
            min_steps: None,
            max_steps: None,
            unexplored_paths: BinaryHeap::new(),
            explored_paths: HashMap::new(),
        })
    }
}

impl Map {
    fn set_origin(&mut self, origin: Coordinate) -> Result<()> {
        self.nodes
            .keys()
            .find(|n| **n == origin)
            .context("Origin is not a node")?;

        let origin_path = PathWithCost {
            total_cost: 0,
            coordinate: origin,
            going_towards: Direction::South,
            going_towards_count: 0,
        };
        self.unexplored_paths.push(origin_path.clone());

        Ok(())
    }
}

impl Map {
    fn explore_smaller_cost_path(&mut self) -> Result<bool> {
        // unexplored_paths is a binary heap ordered by cost descending,
        // meaning that pop will always get the path with smallest cost
        let path_to_explore_with_cost = self.unexplored_paths.pop();

        let path_to_explore_with_cost = match path_to_explore_with_cost {
            Some(p) => p,
            None => {
                // all paths explored
                // returning true informs the consumer to stop looping this function
                return Ok(true);
            }
        };

        let path_to_explore = Path::from(&path_to_explore_with_cost);
        let cost = path_to_explore_with_cost.total_cost;

        if self.explored_paths.contains_key(&path_to_explore) {
            // path has already been explored with a better cost, skipping
            return Ok(false);
        } else {
            self.explored_paths.insert(path_to_explore.clone(), cost);
        }

        let min_steps = self.min_steps.context("Forgot to set min_steps")?;
        let max_steps = self.max_steps.context("Forgot to set max_steps")?;

        'dir: for dir in [
            Direction::North,
            Direction::West,
            Direction::South,
            Direction::East,
        ]
        .iter()
        {
            // don't go back
            if path_to_explore.going_towards.opposite() == *dir {
                continue;
            }

            // if changing directions, walk min_steps
            // if going straight, add 1 until you hit max_steps
            let steps_to_walk = if path_to_explore.going_towards == *dir {
                1
            } else {
                min_steps
            };

            // for each walking step, recalculate the destination coordinate and sum the cost of
            // each travelled tile
            let mut coordinate = path_to_explore.coordinate.clone();
            let mut cost_to_add = 0;
            for _ in 0..steps_to_walk {
                coordinate = coordinate.single_step(&dir);
                cost_to_add += match self.nodes.get(&coordinate) {
                    Some(val) => val,
                    None => {
                        // walking in this direction reaches out of bounds, skipping
                        continue 'dir;
                    }
                };
            }

            // consolidate new path from destination of walking the steps above
            let new_path = Path {
                coordinate,
                going_towards: dir.clone(),
                going_towards_count: if path_to_explore.going_towards == *dir {
                    path_to_explore.going_towards_count + 1
                } else {
                    min_steps
                },
            };

            // add the new path to unexplored unless going straight for more than max_steps
            if !self.explored_paths.contains_key(&new_path)
                && new_path.going_towards_count <= max_steps
            {
                self.unexplored_paths
                    .push(new_path.add_cost(cost + cost_to_add));
            }
        }

        // returning false informs the consumer to keep looping this function
        Ok(false)
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let mut map = Map::try_from(input.trim())?;
    map.set_origin(Coordinate { x: 0, y: 0 })?;
    map.min_steps = Some(1);
    map.max_steps = Some(3);

    let destination_coords = map
        .nodes
        .keys()
        .max_by(|a, b| (a.x + a.y).cmp(&(b.x + b.y)))
        .unwrap()
        .clone();

    loop {
        if map.explore_smaller_cost_path().unwrap() {
            break;
        }
    }

    let result = map
        .explored_paths
        .iter()
        .filter(|(p, _)| p.coordinate == destination_coords)
        .map(|(_, c)| *c)
        .min()
        .unwrap();

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let mut map = Map::try_from(input.trim())?;
    map.set_origin(Coordinate { x: 0, y: 0 })?;
    map.min_steps = Some(4);
    map.max_steps = Some(10);

    let destination_coords = map
        .nodes
        .keys()
        .max_by(|a, b| (a.x + a.y).cmp(&(b.x + b.y)))
        .unwrap()
        .clone();

    loop {
        if map.explore_smaller_cost_path().unwrap() {
            break;
        }
    }

    let result = map
        .explored_paths
        .iter()
        .filter(|(p, _)| p.coordinate == destination_coords)
        .map(|(_, c)| *c)
        .min()
        .unwrap();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d17::run_part_1;
    use crate::d17::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d17/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 102);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d17/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 1244);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d17/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 94);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d17/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 1367);
    }
}
