use std::collections::{HashMap, HashSet};

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
    x: isize,
    y: isize,
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
            Direction::North => x = x.checked_sub(1).unwrap_or(isize::MAX),
            Direction::West => y = y.checked_sub(1).unwrap_or(isize::MAX),
            Direction::South => x = x.checked_add(1).unwrap_or(isize::MAX),
            Direction::East => y = y.checked_add(1).unwrap_or(isize::MAX),
        };

        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Tile {
    StartingPosition,
    GardenPlot,
    Rock,
}

impl TryFrom<char> for Tile {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            'S' => Ok(Tile::StartingPosition),
            '.' => Ok(Tile::GardenPlot),
            '#' => Ok(Tile::Rock),
            _ => Err(anyhow::anyhow!("Invalid tile input")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Map {
    contents: HashMap<Coordinate, Tile>,
    size_x: usize,
    size_y: usize,
    is_infinite: bool,
    steps_taken: usize,
    possible_solutions: HashSet<(Coordinate, Coordinate)>,
}

impl TryFrom<&str> for Map {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut contents = HashMap::new();
        let mut start = None;

        for (i, line) in value.lines().enumerate() {
            for (j, char) in line.chars().enumerate() {
                let coord = Coordinate {
                    x: isize::try_from(i)?,
                    y: isize::try_from(j)?,
                };
                let tile = Tile::try_from(char)?;

                if tile == Tile::StartingPosition {
                    start = Some((Coordinate { x: 0, y: 0 }, coord.clone()));
                }

                contents.insert(coord, tile);
            }
        }

        let size_x = value.lines().count();
        let size_y = value.lines().next().unwrap().chars().count();

        let mut possible_solutions = HashSet::new();
        possible_solutions.insert(start.unwrap());

        Ok(Self {
            contents,
            size_x,
            size_y,
            is_infinite: false,
            steps_taken: 0,
            possible_solutions,
        })
    }
}

impl Map {
    fn take_step(&mut self) {
        let mut new_solutions = HashSet::new();

        for p in self.possible_solutions.iter() {
            Direction::into_iter().for_each(|d| {
                let mut coord = p.1.single_step(&d);
                let mut map_coord = p.0.clone();
                let mut needs_infinite_map = false;
                if coord.x < 0 {
                    needs_infinite_map = true;
                    coord.x += self.size_x as isize;
                    map_coord.x -= 1;
                } else if coord.x >= self.size_x as isize {
                    needs_infinite_map = true;
                    coord.x -= self.size_x as isize;
                    map_coord.x += 1;
                }
                if coord.y < 0 {
                    needs_infinite_map = true;
                    coord.y += self.size_y as isize;
                    map_coord.y -= 1;
                } else if coord.y >= self.size_y as isize {
                    needs_infinite_map = true;
                    coord.y -= self.size_y as isize;
                    map_coord.y += 1;
                }
                if needs_infinite_map && !self.is_infinite {
                    return;
                }

                match self.contents.get(&coord) {
                    Some(&Tile::GardenPlot) | Some(&Tile::StartingPosition) => {
                        new_solutions.insert((map_coord, coord));
                    }
                    Some(&Tile::Rock) => {}
                    None => unreachable!(),
                }
            });
        }

        self.steps_taken += 1;
        self.possible_solutions = new_solutions;
    }
}

/**
 * The trivial solution of navigating step by step and writing the possible solutions is too slow.
 *
 * There is a more intelligent way of knowing if a tile is a possible solution:
 * A tile is always in an even or an odd number of steps from the starting position.
 * If row_tile + col_tile - row_start - col_start is even, this tile can never be a solution if the
 * number of steps to walk is odd, and vice-versa.
 *
 * For a big enough number of steps, we will eventually explore the entire map, and then each step
 * will be switching from all accessible even tiles to all accessible odd tiles, and then back.
 *
 * For the infinite map and a big enough number of steps, we will get multiple map instances fully
 * explored, plus some not fully explored ones. Representing each map instance with a character, it
 * will look like a bigger version of this:
 *
 *       V
 *      BCB
 *     BSOSB
 *    BSOIOSB
 *   BSOIOIOSB
 *  BSOIOIOIOSB
 * VCOIOIOIOIOCV
 *  BSOIOIOIOSB
 *   BSOIOIOSB
 *    BSOIOSB
 *     BSOSB
 *      BCB
 *       V
 *
 * O and I: fully explored map instances, but alternate between even and odd paths if map size
 * is odd.
 * S: non-fully explored map instances near the edge of the diamond
 * B: non-fully explored map instances at the edge of the diamond
 * C: non-fully explored map instances near to the vertex of the diamond
 * V: non-fully explored map instances at the vertex of the diamond
 *
 * The number of solutions for O and I will be the same regardless of the quadrant, but the number
 * of solutions for V C B and S need to be checked 4 times each.
 *
 * The smallest pattern that has at least one of each type is:
 *
 *    V
 *   BCB
 *  BSISB
 * VCIOICV
 *  BSISB
 *   BCB
 *    V
 *
 * The given map has free pathways on edges and on the middle row and column, which asserts that
 * the 4 sides of the diamond will grow equally as the number of steps increases, facilitating the
 * calculation. Thus, we need to walk at least 2.5 times the number of steps. We also must stop at
 * a very specific number of steps to make sure that the edges and the vertices are filled exactly
 * like they will in the version with the bigger steps.
 *
 * The number of solutions for each map instance will be found from its representant in the smaller
 * sample:
 * (0,0) -> O
 * (-1,0) or (1,0) or (0,-1) or (0,1) -> I
 * (-1, -1) -> S-northwest
 * (1, -1) -> S-northeast
 * (-1, 1) -> S-southwest
 * (1, 1) -> S-southeast
 * (-2, -1) or (-1, -2) -> B-northwest
 * (2, -1) or (1, -2) -> B-northeast
 * (-2, 1) or (-1, 2) -> B-southwest
 * (2, 1) or (1, 2) -> B-southeast
 * (-2, 0) -> C-north
 * (0, -2) -> C-west
 * (2, 0) -> C-south
 * (0, 2) -> C-east
 * (-3, 0) -> V-north
 * (0, -3) -> V-west
 * (3, 0) -> V-south
 * (0, 3) -> V-east
 *
 * We then need to multiply the number of occurrences of each map instance to get the value for the
 * bigger input.
 */

pub fn run_part_1(input: String, steps: usize) -> Result<usize> {
    let mut map = Map::try_from(input.trim())?;

    while map.steps_taken < steps {
        map.take_step();
    }

    Ok(map.possible_solutions.len())
}

pub fn run_part_2(input: String, steps: usize) -> Result<usize> {
    let mut map = Map::try_from(input.trim())?;

    map.is_infinite = true;

    let step_limit = {
        let mut result = 2 * map.size_x + map.size_x / 2;
        loop {
            if steps < result || steps % map.size_x == result % map.size_x {
                break;
            }
            result += 1;
        }
        result.min(steps)
    };

    while map.steps_taken < step_limit {
        map.take_step();
    }

    let solution = if map.steps_taken != steps {
        let solutions_per_map_instance =
            map.possible_solutions
                .iter()
                .fold(HashMap::new(), |mut acc, (mc, _c)| {
                    acc.entry(mc).and_modify(|v| *v += 1).or_insert(1);
                    acc
                });

        let diamond_radius = (steps - map.size_x / 2) / map.size_x + 1;

        0
        // O
        + (diamond_radius / 2 * 2 - 1).pow(2)
            * solutions_per_map_instance
                .get(&Coordinate { x: 0, y: 0 })
                .unwrap_or(&0)
        // I
        + ((diamond_radius - 1) / 2 * 2).pow(2)
            * solutions_per_map_instance
                .get(&Coordinate { x: 1, y: 0 })
                .unwrap_or(&0)
        // S
        + (diamond_radius - 2)
            * (solutions_per_map_instance
                .get(&Coordinate { x: -1, y: -1 })
                .unwrap_or(&0)
                + solutions_per_map_instance
                    .get(&Coordinate { x: 1, y: -1 })
                    .unwrap_or(&0)
                + solutions_per_map_instance
                    .get(&Coordinate { x: -1, y: 1 })
                    .unwrap_or(&0)
                + solutions_per_map_instance
                    .get(&Coordinate { x: 1, y: 1 })
                    .unwrap_or(&0))
        // B
        + (diamond_radius - 1)
            * (solutions_per_map_instance
                .get(&Coordinate { x: -2, y: -1 })
                .unwrap_or(&0)
                + solutions_per_map_instance
                    .get(&Coordinate { x: 2, y: -1 })
                    .unwrap_or(&0)
                + solutions_per_map_instance
                    .get(&Coordinate { x: -2, y: 1 })
                    .unwrap_or(&0)
                + solutions_per_map_instance
                    .get(&Coordinate { x: 2, y: 1 })
                    .unwrap_or(&0))
        // C
        + (solutions_per_map_instance
            .get(&Coordinate { x: -2, y: 0 })
            .unwrap_or(&0)
        + solutions_per_map_instance
            .get(&Coordinate { x: 0, y: -2 })
            .unwrap_or(&0)
        + solutions_per_map_instance
            .get(&Coordinate { x: 2, y: 0 })
            .unwrap_or(&0)
        + solutions_per_map_instance
            .get(&Coordinate { x: 0, y: 2 })
            .unwrap_or(&0))
        // V
        + (solutions_per_map_instance
            .get(&Coordinate { x: -3, y: 0 })
            .unwrap_or(&0)
        + solutions_per_map_instance
            .get(&Coordinate { x: 0, y: -3 })
            .unwrap_or(&0)
        + solutions_per_map_instance
            .get(&Coordinate { x: 3, y: 0 })
            .unwrap_or(&0)
        + solutions_per_map_instance
            .get(&Coordinate { x: 0, y: 3 })
            .unwrap_or(&0))
    } else {
        map.possible_solutions.len()
    };

    Ok(solution)
}

#[cfg(test)]
mod tests {
    use crate::d21::run_part_1;
    use crate::d21::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d21/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input, 6).unwrap(), 16);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d21/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input, 64).unwrap(), 3660);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d21/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input, 10).unwrap(), 50);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d21/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input, 26501365).unwrap(), 605492675373144);
    }
}
