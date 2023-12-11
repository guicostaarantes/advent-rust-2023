use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Example {
    example: usize
}

pub fn run_part_1(input: String) -> Result<usize> {
    todo!()
}

pub fn run_part_2(input: String) -> Result<usize> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::tpl::run_part_1;
    use crate::tpl::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/tpl/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), todo!());
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/tpl/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), todo!());
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/tpl/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), todo!());
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/tpl/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), todo!());
    }
}
