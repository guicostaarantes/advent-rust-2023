use std::collections::BTreeMap;

use anyhow::{Context, Result};

#[derive(Clone, PartialEq, Eq)]
enum Tile {
    Ground,
    NorthToSouth,
    EastToWest,
    NorthToEast,
    NorthToWest,
    SouthToWest,
    SouthToEast,
    StartingPosition,
}

impl TryFrom<char> for Tile {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            '.' => Ok(Self::Ground),
            '|' => Ok(Self::NorthToSouth),
            '-' => Ok(Self::EastToWest),
            'L' => Ok(Self::NorthToEast),
            'J' => Ok(Self::NorthToWest),
            '7' => Ok(Self::SouthToWest),
            'F' => Ok(Self::SouthToEast),
            'S' => Ok(Self::StartingPosition),
            _ => Err(anyhow::anyhow!("Invalid character")),
        }
    }
}

#[derive(PartialEq)]
enum Direction {
    East,
    North,
    West,
    South,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Coordinate {
    lattitude: usize,
    longitude: usize,
}

impl Coordinate {
    fn find_by_direction(&self, dir: &Direction) -> Self {
        match dir {
            Direction::East => Self {
                lattitude: self.lattitude,
                longitude: self.longitude + 1,
            },
            Direction::North => Self {
                lattitude: self.lattitude + 1,
                longitude: self.longitude,
            },
            Direction::West => Self {
                lattitude: self.lattitude,
                longitude: self.longitude - 1,
            },
            Direction::South => Self {
                lattitude: self.lattitude - 1,
                longitude: self.longitude,
            },
        }
    }
}

struct Map {
    coordinates: BTreeMap<Coordinate, Tile>,
    size: (usize, usize),
}

impl TryFrom<&str> for Map {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut coordinates = BTreeMap::new();

        let no_of_lines = value.lines().fold(0, |acc, _| acc + 1);
        let no_of_cols = value
            .lines()
            .next()
            .unwrap()
            .chars()
            .fold(0, |acc, _| acc + 1);

        for (i, line) in value.lines().enumerate() {
            for (j, char) in line.chars().enumerate() {
                let tile = Tile::try_from(char)?;
                if tile != Tile::Ground {
                    coordinates.insert(
                        Coordinate {
                            lattitude: no_of_lines - i - 1,
                            longitude: j,
                        },
                        tile,
                    );
                }
            }
        }

        Ok(Self {
            coordinates,
            size: (no_of_lines, no_of_cols),
        })
    }
}

impl Map {
    fn initial_coordinates(&self) -> Result<Coordinate> {
        match self
            .coordinates
            .iter()
            .find(|(_, v)| **v == Tile::StartingPosition)
        {
            Some(co) => Ok(co.0.clone()),
            None => Err(anyhow::anyhow!("Starting position not found")),
        }
    }
}

struct LoopFinder<'a> {
    map: &'a Map,
    current_position: Coordinate,
    current_direction: Direction,
    loop_coordinates: BTreeMap<Coordinate, Tile>,
}

impl<'a> LoopFinder<'a> {
    fn new(map: &'a Map) -> Result<Self> {
        let current_position = map.initial_coordinates()?;

        let result = Self {
            map,
            current_position,
            current_direction: Direction::North,
            loop_coordinates: BTreeMap::new(),
        };

        Ok(result)
    }
}

impl LoopFinder<'_> {
    fn navigate(&mut self) -> Result<bool> {
        let (current_coordinate, current_tile) = self
            .map
            .coordinates
            .get_key_value(&self.current_position)
            .context("Coordinate not found")?;

        match current_tile {
            Tile::Ground => {
                return Err(anyhow::anyhow!("Tile not in loop"));
            }
            Tile::StartingPosition => {
                if self.loop_coordinates.len() > 0 {
                    return Ok(false);
                }

                let east = self
                    .map
                    .coordinates
                    .get(&current_coordinate.find_by_direction(&Direction::East));
                if east == Some(&Tile::EastToWest)
                    || east == Some(&Tile::NorthToWest)
                    || east == Some(&Tile::SouthToWest)
                {
                    self.current_direction = Direction::East;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), east.unwrap().clone());
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                    return Ok(true);
                }

                let north = self
                    .map
                    .coordinates
                    .get(&current_coordinate.find_by_direction(&Direction::North));
                if north == Some(&Tile::NorthToSouth)
                    || north == Some(&Tile::SouthToWest)
                    || north == Some(&Tile::SouthToEast)
                {
                    self.current_direction = Direction::North;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), north.unwrap().clone());
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                    return Ok(true);
                }

                let west = self
                    .map
                    .coordinates
                    .get(&current_coordinate.find_by_direction(&Direction::West));
                if west == Some(&Tile::EastToWest)
                    || west == Some(&Tile::SouthToEast)
                    || west == Some(&Tile::SouthToWest)
                {
                    self.current_direction = Direction::West;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), west.unwrap().clone());
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                    return Ok(true);
                }

                // a valid loop needs at least two entrances, so we expect to have found at least
                // one ending of it.
                return Err(anyhow::anyhow!("Invalid loop"));
            }
            Tile::NorthToSouth => {
                if self.current_direction == Direction::North {
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::NorthToSouth);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else if self.current_direction == Direction::South {
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::NorthToSouth);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else {
                    return Err(anyhow::anyhow!("Invalid loop 1"));
                }
            }
            Tile::EastToWest => {
                if self.current_direction == Direction::East {
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::EastToWest);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else if self.current_direction == Direction::West {
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::EastToWest);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else {
                    return Err(anyhow::anyhow!("Invalid loop 2"));
                }
            }
            Tile::NorthToEast => {
                if self.current_direction == Direction::West {
                    self.current_direction = Direction::North;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::NorthToEast);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else if self.current_direction == Direction::South {
                    self.current_direction = Direction::East;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::NorthToEast);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else {
                    return Err(anyhow::anyhow!("Invalid loop 3"));
                }
            }
            Tile::NorthToWest => {
                if self.current_direction == Direction::South {
                    self.current_direction = Direction::West;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::NorthToWest);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else if self.current_direction == Direction::East {
                    self.current_direction = Direction::North;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::NorthToWest);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else {
                    return Err(anyhow::anyhow!("Invalid loop 4"));
                }
            }
            Tile::SouthToWest => {
                if self.current_direction == Direction::East {
                    self.current_direction = Direction::South;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::SouthToWest);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else if self.current_direction == Direction::North {
                    self.current_direction = Direction::West;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::SouthToWest);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else {
                    return Err(anyhow::anyhow!("Invalid loop 5"));
                }
            }
            Tile::SouthToEast => {
                if self.current_direction == Direction::West {
                    self.current_direction = Direction::South;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::SouthToEast);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else if self.current_direction == Direction::North {
                    self.current_direction = Direction::East;
                    self.loop_coordinates
                        .insert(self.current_position.clone(), Tile::SouthToEast);
                    self.current_position =
                        current_coordinate.find_by_direction(&self.current_direction);
                } else {
                    return Err(anyhow::anyhow!("Invalid loop 6"));
                }
            }
        };

        Ok(true)
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let map = Map::try_from(input.trim())?;
    let mut loop_finder = LoopFinder::new(&map)?;
    while loop_finder.navigate()? {}

    Ok(loop_finder.loop_coordinates.len() / 2)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let map = Map::try_from(input.trim())?;
    let map_size = (map.size.0, map.size.1);
    let mut loop_finder = LoopFinder::new(&map)?;
    while loop_finder.navigate()? {}

    let mut result = 0;
    for i in 0..map_size.0 {
        let mut inside = false;
        let mut lvt: Option<&Tile> = None;
        for j in 0..map_size.1 {
            let coord = Coordinate {
                lattitude: i,
                longitude: j,
            };
            match loop_finder.loop_coordinates.get(&coord) {
                Some(t) => {
                    if t == &Tile::NorthToSouth {
                        inside = !inside;
                    }
                    if t == &Tile::SouthToWest && lvt == Some(&Tile::NorthToEast) {
                        inside = !inside;
                    }
                    if t == &Tile::NorthToWest && lvt == Some(&Tile::SouthToEast) {
                        inside = !inside;
                    }
                    if t != &Tile::EastToWest {
                        lvt = Some(t);
                    }
                }
                None => {
                    if inside {
                        result += 1;
                    }
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d10::run_part_1;
    use crate::d10::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d10/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 8);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d10/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 6828);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d10/test2.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 8);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d10/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 459);
    }
}
