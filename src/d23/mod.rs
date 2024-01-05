use std::collections::{HashMap, VecDeque};

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    fn into_iter() -> std::array::IntoIter<Direction, 4> {
        [
            Direction::North,
            Direction::West,
            Direction::South,
            Direction::East,
        ]
        .into_iter()
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Tile {
    Ground,
    Forest,
    SlopeNorth,
    SlopeWest,
    SlopeSouth,
    SlopeEast,
}

impl TryFrom<char> for Tile {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            '.' => Ok(Tile::Ground),
            '#' => Ok(Tile::Forest),
            '^' => Ok(Tile::SlopeNorth),
            '<' => Ok(Tile::SlopeWest),
            'v' => Ok(Tile::SlopeSouth),
            '>' => Ok(Tile::SlopeEast),
            _ => Err(anyhow::anyhow!("Invalid tile input")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Path {
    from: Coordinate,
    to: Coordinate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Map {
    contents: HashMap<Coordinate, Tile>,
    paths: HashMap<Path, usize>,
    start: Coordinate,
    end: Coordinate,
}

impl TryFrom<&str> for Map {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut contents = HashMap::new();

        for (i, line) in value.lines().enumerate() {
            for (j, char) in line.chars().enumerate() {
                contents.insert(Coordinate { x: i, y: j }, Tile::try_from(char)?);
            }
        }

        let start = contents
            .iter()
            .find(|(co, ti)| co.x == 0 && **ti == Tile::Ground)
            .context("No start")?
            .0
            .clone();

        let end = contents
            .iter()
            .find(|(co, ti)| co.x == value.lines().count() - 1 && **ti == Tile::Ground)
            .context("No end")?
            .0
            .clone();

        Ok(Self {
            contents,
            paths: HashMap::new(),
            start,
            end,
        })
    }
}

impl Map {
    fn can_walk(&self, from: &Coordinate, to: &Direction) -> bool {
        match self.contents.get(from) {
            Some(&Tile::Ground) => match self.contents.get(&from.single_step(to)) {
                Some(&Tile::Ground) => true,
                Some(&Tile::SlopeNorth) => *to != Direction::South,
                Some(&Tile::SlopeWest) => *to != Direction::East,
                Some(&Tile::SlopeSouth) => *to != Direction::North,
                Some(&Tile::SlopeEast) => *to != Direction::West,
                _ => false,
            },
            Some(&Tile::SlopeNorth) => *to == Direction::North,
            Some(&Tile::SlopeWest) => *to == Direction::West,
            Some(&Tile::SlopeSouth) => *to == Direction::South,
            Some(&Tile::SlopeEast) => *to == Direction::East,
            Some(&Tile::Forest) => false,
            _ => unreachable!(),
        }
    }
}

impl Map {
    fn build_paths(&mut self) {
        // find tiles with 3 or 4 connections
        let intersections = self
            .contents
            .iter()
            .filter(|(co, ti)| {
                **co == self.start
                    || **co == self.end
                    || Direction::into_iter()
                        .filter(|d| {
                            ti != &&Tile::Forest
                                && match self.contents.get(&co.single_step(&d)) {
                                    Some(&Tile::Forest) => false,
                                    Some(_) => true,
                                    None => false,
                                }
                        })
                        .count()
                        > 2
            })
            .map(|(co, _)| co.clone())
            .collect::<Vec<Coordinate>>();

        // for each intersection, explore until you find other intersections
        for i in intersections.iter() {
            let mut exploring_hikes = VecDeque::new();
            exploring_hikes.push_back(vec![i.clone()]);
            while let Some(h) = exploring_hikes.pop_front() {
                let current = h.iter().last().unwrap();
                // explore all 4 directions
                for d in Direction::into_iter() {
                    // only explore if you can walk in that direction
                    if self.can_walk(&current, &d) {
                        let next = current.single_step(&d);
                        // only explore if it's never explored before in this hike
                        if !h.contains(&next) {
                            // if next step is intersection, add path between the two
                            if intersections.contains(&next) {
                                self.paths
                                    .entry(Path {
                                        from: i.clone(),
                                        to: next,
                                    })
                                    .and_modify(|v| {
                                        *v = usize::max(*v, h.len());
                                    })
                                    .or_insert(h.len());
                            } else {
                                // continue exploring until hits a dead end or an intersection
                                exploring_hikes.push_back([h.clone(), vec![next]].concat());
                            }
                        }
                    }
                }
            }
        }

        // TODO: reduce number of paths by finding pairs where starting from the start of the first
        // path always lead to the end of the second path, then simplifying them to be a single path
        // with distance equal to the largest distance between paths in the subproblem.
    }
}

impl Map {
    fn find_largest_path(&self) -> usize {
        let mut hikes_from_start_to_end = Vec::new();
        let mut exploring_hikes = VecDeque::new();
        exploring_hikes.push_back(vec![self.start.clone()]);

        while let Some(h) = exploring_hikes.pop_front() {
            let current = h.iter().last().unwrap();
            self.paths
                .iter()
                .filter(|(n, _)| n.from == *current)
                .for_each(|(n, _)| {
                    if n.to == self.end {
                        hikes_from_start_to_end.push([h.clone(), vec![n.to.clone()]].concat());
                    } else if !h.contains(&n.to) {
                        exploring_hikes.push_back([h.clone(), vec![n.to.clone()]].concat());
                    }
                });
        }

        let max_distance = hikes_from_start_to_end
            .iter()
            .map(|h| {
                let mut distance = 0;
                (1..h.len()).for_each(|i| {
                    distance += self
                        .paths
                        .get(&Path {
                            from: h[i - 1].clone(),
                            to: h[i].clone(),
                        })
                        .unwrap()
                });
                distance
            })
            .max();

        max_distance.unwrap()
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let mut map = Map::try_from(input.trim())?;

    map.build_paths();

    Ok(map.find_largest_path())
}

pub fn run_part_2(input: String) -> Result<usize> {
    let mut map = Map::try_from(
        input
            .trim()
            .replace("^", ".")
            .replace("<", ".")
            .replace("v", ".")
            .replace(">", ".")
            .as_str(),
    )?;

    map.build_paths();

    Ok(map.find_largest_path())
}

#[cfg(test)]
mod tests {
    use crate::d23::run_part_1;
    use crate::d23::run_part_2;
    use std::fs::read_to_string;

    /* #[test]
    fn part_1_test() {
        let input = read_to_string("src/d23/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 94);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d23/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 2106);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d23/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 154);
    } */

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d23/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 6350);
    }
}
