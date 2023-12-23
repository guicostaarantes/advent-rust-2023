use anyhow::{Context, Result};

#[derive(Clone)]
struct EqualInstruction {
    label: String,
    focus: usize,
    label_hash: usize,
    raw_hash: usize,
}

#[derive(Clone)]
struct DashInstruction {
    label: String,
    label_hash: usize,
    raw_hash: usize,
}

#[derive(Clone)]
enum Instruction {
    Equal(EqualInstruction),
    Dash(DashInstruction),
}

enum InstructionType {
    Equal,
    Dash,
}

impl TryFrom<&str> for Instruction {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut instruction_type = InstructionType::Equal;

        let (label, focus_str) = value
            .split_once("=")
            .or_else(|| {
                instruction_type = InstructionType::Dash;
                value.split_once("-")
            })
            .context("bad")?;

        let label = label.to_string();

        let mut focus = 0;
        if focus_str != "" {
            focus = focus_str.parse::<usize>()?;
        }

        let mut label_hash = 0;
        let mut raw_hash = 0;

        value.chars().map(|c| c as usize).for_each(|n| {
            if n == 45 {
                instruction_type = InstructionType::Dash;
                label_hash = raw_hash;
            } else if n == 61 {
                instruction_type = InstructionType::Equal;
                label_hash = raw_hash;
            }

            raw_hash += n;
            raw_hash = 17 * raw_hash;
            raw_hash = raw_hash % 256;
        });

        match instruction_type {
            InstructionType::Equal => Ok(Self::Equal(EqualInstruction {
                label,
                focus,
                label_hash,
                raw_hash,
            })),
            InstructionType::Dash => Ok(Self::Dash(DashInstruction {
                label,
                label_hash,
                raw_hash,
            })),
        }
    }
}

#[derive(Debug)]
struct Lens {
    label: String,
    focus: usize,
}

impl From<&EqualInstruction> for Lens {
    fn from(value: &EqualInstruction) -> Self {
        Self {
            label: value.label.clone(),
            focus: value.focus,
        }
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let raw_hashes = input
        .trim()
        .split(',')
        .map(|i| Instruction::try_from(i))
        .map(|h| match h {
            Ok(Instruction::Equal(e)) => Ok(e.raw_hash),
            Ok(Instruction::Dash(d)) => Ok(d.raw_hash),
            Err(_) => Err(anyhow::anyhow!("Bad input")),
        })
        .collect::<Result<Vec<usize>>>()?;

    Ok(raw_hashes.iter().sum())
}

pub fn run_part_2(input: String) -> Result<usize> {
    let mut boxes: Vec<Vec<Lens>> = Vec::new();
    for _ in 0..256 {
        boxes.push(Vec::new());
    }

    let instructions = input
        .trim()
        .split(',')
        .map(|i| Instruction::try_from(i))
        .collect::<Result<Vec<Instruction>>>()?;

    for i in instructions.iter() {
        match i {
            Instruction::Equal(e) => {
                match boxes[e.label_hash].iter_mut().find(|l| e.label == l.label) {
                    Some(l) => {
                        l.focus = e.focus;
                    }
                    None => {
                        boxes[e.label_hash].push(Lens::from(e));
                    }
                }
            }
            Instruction::Dash(d) => {
                boxes[d.label_hash].retain(|l| l.label != d.label);
            }
        }
    }

    let mut result = 0;

    for (bn, b) in boxes.iter().enumerate() {
        for (ln, l) in b.iter().enumerate() {
            result += (bn + 1) * (ln + 1) * l.focus
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d15::run_part_1;
    use crate::d15::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d15/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 1320);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d15/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 510792);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d15/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 145);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d15/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 269410);
    }
}
