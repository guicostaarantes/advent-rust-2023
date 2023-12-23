use anyhow::{Context, Result};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Coordinate {
    x: usize,
    y: usize,
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

impl Coordinate {
    fn single_step(&self, dir: &Direction) -> Self {
        let mut x = self.x;
        let mut y = self.y;

        match dir {
            Direction::Up => x = x.checked_sub(1).unwrap_or(usize::MAX),
            Direction::Left => y = y.checked_sub(1).unwrap_or(usize::MAX),
            Direction::Down => x = x.checked_add(1).unwrap_or(usize::MAX),
            Direction::Right => y = y.checked_add(1).unwrap_or(usize::MAX),
        };

        Self { x, y }
    }
}

enum InstructionType {
    DirAndSteps,
    Color,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl TryFrom<(&str, &InstructionType)> for Direction {
    type Error = anyhow::Error;

    fn try_from((value, typ): (&str, &InstructionType)) -> Result<Self> {
        match typ {
            InstructionType::DirAndSteps => match value {
                "U" => Ok(Direction::Up),
                "L" => Ok(Direction::Left),
                "D" => Ok(Direction::Down),
                "R" => Ok(Direction::Right),
                _ => Err(anyhow::anyhow!("Invalid direction")),
            },
            InstructionType::Color => match value {
                "3" => Ok(Direction::Up),
                "2" => Ok(Direction::Left),
                "1" => Ok(Direction::Down),
                "0" => Ok(Direction::Right),
                _ => Err(anyhow::anyhow!("Invalid direction")),
            },
        }
    }
}

struct Instruction {
    direction: Direction,
    steps: usize,
}

impl TryFrom<(&str, &InstructionType)> for Instruction {
    type Error = anyhow::Error;

    fn try_from((value, typ): (&str, &InstructionType)) -> Result<Self> {
        match typ {
            InstructionType::DirAndSteps => {
                let (direction, rest) = value.split_once(" ").context("Bad input")?;
                let (steps, _) = rest.split_once(" (#").context("Bad input")?;

                let direction = Direction::try_from((direction, typ)).context("Bad input")?;
                let steps = steps.parse::<usize>().context("Bad input")?;

                Ok(Self { direction, steps })
            }
            InstructionType::Color => {
                let (_, color) = value.split_once(" (#").context("Bad input")?;
                let (color, _) = color.split_once(")").context("Bad input")?;

                let (steps, direction) = color.split_at(5);
                let steps = usize::from_str_radix(steps, 16).context("Bad input")?;
                let direction = Direction::try_from((direction, typ)).context("Bad input")?;

                Ok(Self { direction, steps })
            }
        }
    }
}

struct Plan {
    instructions: Vec<Instruction>,
}

impl TryFrom<(&str, &InstructionType)> for Plan {
    type Error = anyhow::Error;

    fn try_from((value, typ): (&str, &InstructionType)) -> Result<Self> {
        let instructions = value
            .lines()
            .map(|line| Instruction::try_from((line, typ)))
            .collect::<Result<Vec<Instruction>>>()?;

        Ok(Self { instructions })
    }
}

struct Map {
    vertices: Vec<Coordinate>,
    size: Coordinate,
    area: f64,
}

impl std::fmt::Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in 0..=self.size.x {
            write!(f, "\n")?;
            for y in 0..=self.size.y {
                if self.vertices.contains(&Coordinate { x, y }) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
        }
        write!(f, "\n")
    }
}

impl Map {
    fn from_plan(plan: &Plan) -> Self {
        let mut vertices = Vec::new();
        let mut perimeter = 0;

        let mut current_position = Coordinate {
            x: usize::MAX / 2,
            y: usize::MAX / 2,
        };
        vertices.push(current_position.clone());

        // fill vertices and calculate perimeter
        for ins in plan.instructions.iter() {
            for _ in 0..ins.steps {
                current_position = current_position.single_step(&ins.direction);
                perimeter += 1;
            }
            vertices.push(current_position.clone());
        }

        // adjust map so that min coordinates are 0 for x and y
        let min_x = vertices.iter().min_by(|a, b| a.x.cmp(&b.x)).unwrap().x;
        let min_y = vertices.iter().min_by(|a, b| a.y.cmp(&b.y)).unwrap().y;
        let vertices = vertices
            .iter()
            .map(|d| Coordinate {
                x: d.x - min_x,
                y: d.y - min_y,
            })
            .collect::<Vec<Coordinate>>();

        // calculate size
        let max_x = vertices.iter().max_by(|a, b| a.x.cmp(&b.x)).unwrap().x;
        let max_y = vertices.iter().max_by(|a, b| a.y.cmp(&b.y)).unwrap().y;
        let size = Coordinate { x: max_x, y: max_y };

        // shoelace formula to find the area
        let mut area = 0.;
        for k in 0..vertices.len() - 1 {
            let x1 = vertices[k].x as f64;
            let y1 = vertices[k].y as f64;
            let x2 = vertices[k + 1].x as f64;
            let y2 = vertices[k + 1].y as f64;
            area += x1 * y2 - x2 * y1;
        }
        area = (area / 2.).abs();
        dbg!(&area, &perimeter);

        // the shoelace formula is calculating the area from the center of each tile, but the area
        // should cover the entire tile, so we need to add 0.5m2 per tile in the perimeter that per
        // tile in the perimeter
        area += perimeter as f64 / 2.;

        // there is also the need to add 1 m2 due to cover for the 360 degress of uncovered area
        // that adds to all vertices of a polygon
        area += 1.;

        Self {
            vertices,
            size,
            area,
        }
    }
}

pub fn run_part_1(input: String) -> Result<f64> {
    let plan = Plan::try_from((input.trim(), &InstructionType::DirAndSteps))?;

    let map = Map::from_plan(&plan);

    Ok(map.area)
}

pub fn run_part_2(input: String) -> Result<f64> {
    let plan = Plan::try_from((input.trim(), &InstructionType::Color))?;

    let map = Map::from_plan(&plan);

    Ok(map.area)
}

#[cfg(test)]
mod tests {
    use crate::d18::run_part_1;
    use crate::d18::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d18/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 62.);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d18/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 108909.);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d18/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 952408144115.);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d18/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 133125706867777.);
    }
}
