use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Pixel {
    Off,
    On,
}

impl TryFrom<char> for Pixel {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            '.' => Ok(Pixel::Off),
            '#' => Ok(Pixel::On),
            _ => Err(anyhow::anyhow!("Invalid pixel")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Line {
    pixels: Vec<Pixel>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Image {
    rows: Vec<Line>,
    cols: Vec<Line>,
}

impl TryFrom<&str> for Image {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let rows = value
            .lines()
            .map(|line| {
                let pixels = line
                    .chars()
                    .map(|char| Pixel::try_from(char))
                    .collect::<Result<Vec<Pixel>>>()
                    .context("Bad input")?;

                Ok(Line { pixels })
            })
            .collect::<Result<Vec<Line>>>()?;

        // transpose rows to get columns
        let cols = (0..rows[0].pixels.len())
            .map(|col| {
                let pixels = (0..rows.len())
                    .map(|row| rows[row].pixels[col].clone())
                    .collect::<Vec<Pixel>>();

                Line { pixels }
            })
            .collect::<Vec<Line>>();

        Ok(Self { rows, cols })
    }
}

impl Image {
    fn is_horizontal_mirror(&self, index: usize) -> bool {
        let compare = std::cmp::min(self.rows.len() - index, index);

        for k in 0..compare {
            if self.rows[index - k - 1] != self.rows[index + k] {
                return false;
            }
        }

        true
    }

    fn is_vertical_mirror(&self, index: usize) -> bool {
        let compare = std::cmp::min(self.cols.len() - index, index);

        for k in 0..compare {
            if self.cols[index - k - 1] != self.cols[index + k] {
                return false;
            }
        }

        true
    }

    fn find_all_mirrors(&self, stop_on_first_find: bool) -> Vec<usize> {
        let mut result = Vec::new();

        for r in 1..self.rows.len() {
            if self.is_horizontal_mirror(r) {
                result.push(100 * r);
                if stop_on_first_find {
                    break;
                }
            }
        }

        if !(stop_on_first_find && result.len() > 0) {
            for c in 1..self.cols.len() {
                if self.is_vertical_mirror(c) {
                    result.push(c);
                }
            }
        }

        result
    }

    fn switch_pixel_at(&mut self, row: usize, col: usize) {
        if let Some(row) = self.rows.get_mut(row) {
            if let Some(cell) = row.pixels.get_mut(col) {
                *cell = if cell == &Pixel::Off {
                    Pixel::On
                } else {
                    Pixel::Off
                };
            }
        }
        if let Some(col) = self.cols.get_mut(col) {
            if let Some(cell) = col.pixels.get_mut(row) {
                *cell = if cell == &Pixel::Off {
                    Pixel::On
                } else {
                    Pixel::Off
                };
            }
        }
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let images = input
        .trim()
        .split("\n\n")
        .map(|s| Image::try_from(s))
        .collect::<Result<Vec<Image>>>()?;

    let mut result = 0;

    for i in images.iter() {
        result += i.find_all_mirrors(true)[0];
    }

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let mut images = input
        .trim()
        .split("\n\n")
        .map(|s| Image::try_from(s))
        .collect::<Result<Vec<Image>>>()?;

    let mut result = 0;

    for i in images.iter_mut() {
        // calculating result before removing smudge for comparison
        let old_result = i.find_all_mirrors(true)[0];

        // finding smudge via brute force
        'outer: for r in 0..i.rows.len() {
            for c in 0..i.cols.len() {
                i.switch_pixel_at(r, c);

                let all_mirrors = i.find_all_mirrors(false);

                let other_mirror = all_mirrors
                    .iter()
                    .find(|mi| **mi != old_result);

                if let Some(new_result) = other_mirror {
                    result += new_result;
                    break 'outer;
                }

                i.switch_pixel_at(r, c);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d13::run_part_1;
    use crate::d13::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d13/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 405);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d13/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 33356);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d13/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 400);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d13/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 28475);
    }
}
