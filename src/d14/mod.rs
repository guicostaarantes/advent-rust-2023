use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Cell {
    Empty,
    CubeRock,
    RoundRock,
}

impl TryFrom<char> for Cell {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            '.' => Ok(Self::Empty),
            '#' => Ok(Self::CubeRock),
            'O' => Ok(Self::RoundRock),
            _ => Err(anyhow::anyhow!("Invalid cell")),
        }
    }
}

impl Cell {
    fn to_char(&self) -> char {
        match self {
            Cell::Empty => '.',
            Cell::CubeRock => '#',
            Cell::RoundRock => 'O',
        }
    }
}

#[derive(Debug, Clone)]
struct Grid {
    contents: Vec<Vec<Cell>>,
}

impl TryFrom<&str> for Grid {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let contents = value
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| Cell::try_from(c))
                    .collect::<Result<Vec<Cell>>>()
            })
            .collect::<Result<Vec<Vec<Cell>>>>()
            .context("Invalid input")?;

        Ok(Self { contents })
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        for i in self.contents.iter() {
            for j in i.iter() {
                result.push(j.to_char());
            }
            result.push('\n');
        }

        write!(f, "{}", result)
    }
}

impl Grid {
    fn rotate_90(&mut self) {
        let rows = self.contents.len();
        let cols = self.contents[0].len();

        let rotated = (0..cols)
            .rev()
            .map(|col| {
                (0..rows)
                    .map(|row| self.contents[row][col].clone())
                    .collect::<Vec<Cell>>()
            })
            .collect::<Vec<Vec<Cell>>>();

        self.contents = rotated;
    }

    fn rotate_270(&mut self) {
        let rows = self.contents.len();
        let cols = self.contents[0].len();

        let rotated = (0..cols)
            .map(|col| {
                (0..rows)
                    .rev()
                    .map(|row| self.contents[row][col].clone())
                    .collect::<Vec<Cell>>()
            })
            .collect::<Vec<Vec<Cell>>>();

        self.contents = rotated;
    }

    fn roll_west(&mut self) {
        for line in self.contents.iter_mut() {
            let mut j = 1;
            loop {
                match line.get(j) {
                    Some(&Cell::RoundRock) => {
                        if let Some(&Cell::Empty) = line.get(j.checked_sub(1).unwrap_or(0)) {
                            line[j - 1] = Cell::RoundRock;
                            line[j] = Cell::Empty;
                            j -= 1;
                        } else {
                            j += 1;
                        }
                    }
                    Some(_) => j += 1,
                    None => break,
                }
            }
        }
    }

    fn calculate_load(&self) -> Vec<usize> {
        let mut result = Vec::new();

        for line in self.contents.iter() {
            let mut line_result = 0;
            for j in 0..line.len() {
                if let Some(&Cell::RoundRock) = line.get(j) {
                    line_result += line.len() - j;
                }
            }
            result.push(line_result);
        }

        result
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let mut grid = Grid::try_from(input.trim())?;

    grid.rotate_90();
    grid.roll_west();
    let result = grid.calculate_load();

    Ok(result.iter().sum())
}

pub fn run_part_2(input: String) -> Result<usize> {
    let mut grid = Grid::try_from(input.trim())?;

    let mut history: Vec<Vec<usize>> = Vec::new();

    let start_of_cycle;
    let length_of_cycle;

    grid.rotate_90();
    loop {
        for _ in 0..4 {
            grid.roll_west();
            grid.rotate_270();
        }

        let result = grid.calculate_load();

        if let Some(pos) = history.iter().position(|h| *h == result) {
            println!("State of {pos} is equal to state of {}", history.len());
            start_of_cycle = pos;
            length_of_cycle = history.len() - pos;
            break;
        } else {
            history.push(result);
        }
    }

    let result =
        &history[start_of_cycle - 1 + ((1_000_000_000 - start_of_cycle) % length_of_cycle)];

    Ok(result.iter().sum())
}

#[cfg(test)]
mod tests {
    use crate::d14::run_part_1;
    use crate::d14::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d14/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 136);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d14/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 110565);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d14/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 64);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d14/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 89845);
    }
}
