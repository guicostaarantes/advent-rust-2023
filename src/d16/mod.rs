use std::collections::BTreeMap;
use std::collections::BTreeSet;

use anyhow::{Context, Result};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Coordinate {
    x: isize,
    y: isize,
}

impl std::fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Co({},{})", self.x, self.y)
    }
}

impl std::ops::AddAssign for Coordinate {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Beam {
    start: Coordinate,
    end: Coordinate,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NewBeam {
    start: Coordinate,
    direction: Direction,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    fn to_coord(&self) -> Coordinate {
        match self {
            Direction::North => Coordinate { x: -1, y: 0 },
            Direction::West => Coordinate { x: 0, y: -1 },
            Direction::South => Coordinate { x: 1, y: 0 },
            Direction::East => Coordinate { x: 0, y: 1 },
        }
    }
}

#[derive(Debug, Clone)]
enum Item {
    Empty,
    MirrorLeft,
    MirrorRight,
    SplitterHoriz,
    SplitterVert,
}

impl TryFrom<char> for Item {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            '.' => Ok(Self::Empty),
            '\\' => Ok(Self::MirrorLeft),
            '/' => Ok(Self::MirrorRight),
            '|' => Ok(Self::SplitterVert),
            '-' => Ok(Self::SplitterHoriz),
            _ => Err(anyhow::anyhow!("Invalid item")),
        }
    }
}

#[derive(Clone)]
struct Map {
    contents: BTreeMap<Coordinate, Item>,
    bounds: (Coordinate, Coordinate),
    beams: BTreeSet<Beam>,
    new_beams: BTreeSet<NewBeam>,
    energized_tiles: BTreeSet<Coordinate>,
}

impl std::fmt::Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("map")
            .field("beams", &self.beams)
            .field("new_beams", &self.new_beams)
            .field("energized_tiles", &self.energized_tiles)
            .finish()
    }
}

impl TryFrom<&str> for Map {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut contents = BTreeMap::new();

        value
            .lines()
            .enumerate()
            .map(|(i, line)| {
                line.chars()
                    .enumerate()
                    .map(|(j, char)| {
                        let item = Item::try_from(char).context("Invalid input")?;
                        contents.insert(
                            Coordinate {
                                x: i as isize,
                                y: j as isize,
                            },
                            item,
                        );
                        anyhow::Ok(())
                    })
                    .last();
            })
            .last();

        let mut bounds = (Coordinate { x: 0, y: 0 }, Coordinate { x: 0, y: 0 });
        for k in contents.keys() {
            if k.x > bounds.1.x {
                bounds.1.x = k.x;
            }
            if k.y > bounds.1.y {
                bounds.1.y = k.y;
            }
        }

        let beams = BTreeSet::new();
        let new_beams = BTreeSet::new();
        let energized_tiles = BTreeSet::new();

        Ok(Self {
            contents,
            bounds,
            beams,
            new_beams,
            energized_tiles,
        })
    }
}

impl Map {
    fn reset_state(&mut self) {
        self.beams.clear();
        self.new_beams.clear();
        self.energized_tiles.clear();
    }
}

impl Map {
    fn add_new_beam(&mut self, new_beam: NewBeam) {
        self.new_beams.insert(new_beam);
    }
}

impl Map {
    fn add_beam(&mut self, mut beam: Beam) {
        // keep beam inbounds
        if beam.start.x < self.bounds.0.x {
            beam.start.x = self.bounds.0.x;
        } else if beam.start.x > self.bounds.1.x {
            beam.start.x = self.bounds.1.x;
        }
        if beam.end.x < self.bounds.0.x {
            beam.end.x = self.bounds.0.x;
        } else if beam.end.x > self.bounds.1.x {
            beam.end.x = self.bounds.1.x;
        }
        if beam.start.y < self.bounds.0.y {
            beam.start.y = self.bounds.0.y;
        } else if beam.start.y > self.bounds.1.y {
            beam.start.y = self.bounds.1.y;
        }
        if beam.end.y < self.bounds.0.y {
            beam.end.y = self.bounds.0.y;
        } else if beam.end.y > self.bounds.1.y {
            beam.end.y = self.bounds.1.y;
        }

        if beam.start == beam.end {
            // one tile beam, not needed to count energized_tiles
        } else if beam.start.x == beam.end.x {
            // Vertical beam
            let interval = if beam.start.y < beam.end.y {
                beam.start.y..=beam.end.y
            } else {
                beam.end.y..=beam.start.y
            };
            for k in interval {
                self.energized_tiles.insert(Coordinate {
                    x: beam.start.x,
                    y: k,
                });
            }
            self.beams.insert(beam);
        } else if beam.start.y == beam.end.y {
            // Horizontal beam
            let interval = if beam.start.x < beam.end.x {
                beam.start.x..=beam.end.x
            } else {
                beam.end.x..=beam.start.x
            };
            for k in interval {
                self.energized_tiles.insert(Coordinate {
                    x: k,
                    y: beam.start.y,
                });
            }
            self.beams.insert(beam);
        } else {
            unreachable!();
        }
    }
}

impl Map {
    fn propagate_new_beams(&mut self) {
        let nbs = self.new_beams.clone();

        self.new_beams.clear();

        for nb in nbs.iter() {
            let mut coord = nb.start.clone();

            loop {
                coord += nb.direction.to_coord();
                match self.contents.get(&coord) {
                    Some(Item::MirrorLeft) => {
                        let beam = Beam {
                            start: nb.start.clone(),
                            end: coord.clone(),
                        };

                        if !self.beams.contains(&beam) {
                            self.add_beam(beam);
                            self.new_beams.insert(NewBeam {
                                start: coord,
                                direction: match nb.direction {
                                    Direction::North => Direction::West,
                                    Direction::West => Direction::North,
                                    Direction::South => Direction::East,
                                    Direction::East => Direction::South,
                                },
                            });
                        }

                        break;
                    }
                    Some(Item::MirrorRight) => {
                        let beam = Beam {
                            start: nb.start.clone(),
                            end: coord.clone(),
                        };

                        if !self.beams.contains(&beam) {
                            self.add_beam(beam);
                            self.new_beams.insert(NewBeam {
                                start: coord,
                                direction: match nb.direction {
                                    Direction::North => Direction::East,
                                    Direction::West => Direction::South,
                                    Direction::South => Direction::West,
                                    Direction::East => Direction::North,
                                },
                            });
                        }

                        break;
                    }
                    Some(Item::SplitterHoriz) => match nb.direction {
                        Direction::North | Direction::South => {
                            let beam = Beam {
                                start: nb.start.clone(),
                                end: coord.clone(),
                            };

                            if !self.beams.contains(&beam) {
                                self.add_beam(beam);
                                self.new_beams.insert(NewBeam {
                                    start: coord.clone(),
                                    direction: Direction::West,
                                });
                                self.new_beams.insert(NewBeam {
                                    start: coord,
                                    direction: Direction::East,
                                });
                            }

                            break;
                        }
                        _ => {}
                    },
                    Some(Item::SplitterVert) => match nb.direction {
                        Direction::West | Direction::East => {
                            let beam = Beam {
                                start: nb.start.clone(),
                                end: coord.clone(),
                            };

                            if !self.beams.contains(&beam) {
                                self.add_beam(beam);
                                self.new_beams.insert(NewBeam {
                                    start: coord.clone(),
                                    direction: Direction::North,
                                });
                                self.new_beams.insert(NewBeam {
                                    start: coord,
                                    direction: Direction::South,
                                });
                            }

                            break;
                        }
                        _ => {}
                    },
                    Some(Item::Empty) => {}
                    None => {
                        if coord != nb.start {
                            let beam = Beam {
                                start: nb.start.clone(),
                                end: coord.clone(),
                            };
                            if !self.beams.contains(&beam) {
                                self.add_beam(beam);
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let mut map = Map::try_from(input.trim())?;

    map.add_new_beam(NewBeam {
        start: Coordinate { x: 0, y: -1 },
        direction: Direction::East,
    });

    while map.new_beams.len() > 0 {
        map.propagate_new_beams();
    }

    Ok(map.energized_tiles.len())
}

pub fn run_part_2(input: String) -> Result<usize> {
    let mut map = Map::try_from(input.trim())?;

    let mut result = 0;

    for k in 0..=map.bounds.1.x {
        map.reset_state();

        map.add_new_beam(NewBeam {
            start: Coordinate { x: k, y: -1 },
            direction: Direction::East,
        });

        while map.new_beams.len() > 0 {
            map.propagate_new_beams();
        }

        if map.energized_tiles.len() > result {
            result = map.energized_tiles.len();
        }
    }

    for k in 0..=map.bounds.1.x {
        map.reset_state();

        map.add_new_beam(NewBeam {
            start: Coordinate {
                x: k,
                y: map.bounds.1.x + 1,
            },
            direction: Direction::West,
        });

        while map.new_beams.len() > 0 {
            map.propagate_new_beams();
        }

        if map.energized_tiles.len() > result {
            result = map.energized_tiles.len();
        }
    }

    for k in 0..=map.bounds.1.y {
        map.reset_state();

        map.add_new_beam(NewBeam {
            start: Coordinate { x: -1, y: k },
            direction: Direction::South,
        });

        while map.new_beams.len() > 0 {
            map.propagate_new_beams();
        }

        if map.energized_tiles.len() > result {
            result = map.energized_tiles.len();
        }
    }

    for k in 0..=map.bounds.1.y {
        map.reset_state();

        map.add_new_beam(NewBeam {
            start: Coordinate {
                x: map.bounds.1.y + 1,
                y: k,
            },
            direction: Direction::North,
        });

        while map.new_beams.len() > 0 {
            map.propagate_new_beams();
        }

        if map.energized_tiles.len() > result {
            result = map.energized_tiles.len();
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d16::run_part_1;
    use crate::d16::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d16/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 46);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d16/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 8146);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d16/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 51);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d16/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 8358);
    }
}
