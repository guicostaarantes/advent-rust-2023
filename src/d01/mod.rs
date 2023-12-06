use anyhow::{Context, Result};

pub fn run_part_1(input: String) -> Result<u32> {
    let mut result: u32 = 0;

    let lines = input.trim().split("\n");

    for line in lines {
        let first = line
            .chars()
            .find(|char| char.is_digit(10))
            .context("No digits found")?
            .to_digit(10)
            .unwrap();
        let last = line
            .chars()
            .rev()
            .find(|char| char.is_digit(10))
            .context("No digits found")?
            .to_digit(10)
            .unwrap();

        result = result + 10 * first + last;
    }

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<u32> {
    let mut result: u32 = 0;

    let lines = input.trim().split("\n");

    for line in lines {
        // keeping letters that begin and end numbers, to cover edge cases
        // such as eightwo and nineight
        let convert = line
            .replace("one", "o1e")
            .replace("two", "t2o")
            .replace("three", "t3e")
            .replace("four", "4")
            .replace("five", "5e")
            .replace("six", "6")
            .replace("seven", "7n")
            .replace("eight", "e8t")
            .replace("nine", "n9e");
        let first = convert
            .chars()
            .find(|char| char.is_digit(10))
            .context("No digits found")?
            .to_digit(10)
            .unwrap();
        let last = convert
            .chars()
            .rev()
            .find(|char| char.is_digit(10))
            .context("No digits found")?
            .to_digit(10)
            .unwrap();

        let line_result = 10 * first + last;
        result = result + line_result;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d01::run_part_1;
    use crate::d01::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d01/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 142);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d01/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 54990);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d01/test2.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 281);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d01/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 54473);
    }
}
