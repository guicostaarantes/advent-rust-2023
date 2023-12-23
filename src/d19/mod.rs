use std::collections::{BTreeMap, VecDeque};

use anyhow::{Context, Result};

const MIN_VALUE: usize = 1;
const MAX_VALUE: usize = 4001;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Rating {
    X,
    M,
    A,
    S,
}

impl TryFrom<char> for Rating {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            'x' => Ok(Rating::X),
            'm' => Ok(Rating::M),
            'a' => Ok(Rating::A),
            's' => Ok(Rating::S),
            _ => Err(anyhow::anyhow!("Invalid value")),
        }
    }
}

impl Rating {
    fn into_iter() -> std::array::IntoIter<Rating, 4> {
        [Rating::X, Rating::M, Rating::A, Rating::S].into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Condition {
    rating: Rating,
    signal: char,
    number: usize,
}

impl TryFrom<&str> for Condition {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let signal = ['<', '>']
            .iter()
            .find(|s| value.split_once(**s).is_some())
            .context("Invalid input")?;
        let (rating, number) = value.split_once(*signal).context("Bad input")?;

        let rating = rating.chars().next().context("Bad input")?;
        let rating = Rating::try_from(rating)?;
        let signal = *signal;
        let number = number.parse::<usize>().context("Bad input")?;

        Ok(Self {
            rating,
            signal,
            number,
        })
    }
}

impl Condition {
    fn invert(&self) -> Self {
        match self.signal {
            '<' => Condition {
                rating: self.rating.clone(),
                signal: '>',
                number: self.number - 1,
            },
            '>' => Condition {
                rating: self.rating.clone(),
                signal: '<',
                number: self.number + 1,
            },
            _ => self.clone(),
        }
    }
}

// Each entry of intervals represent (lower limit including, upper limit excluding)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Interval {
    values_per_rating: BTreeMap<Rating, Vec<(usize, usize)>>,
}

impl Interval {
    fn all() -> Self {
        let mut values_per_rating = BTreeMap::new();

        Rating::into_iter().for_each(|r| {
            values_per_rating.insert(r, vec![(MIN_VALUE, MAX_VALUE)]);
        });

        Self { values_per_rating }
    }
}

impl Interval {
    fn contains_part(&self, part: &Part) -> bool {
        Rating::into_iter().all(|r| {
            let val = part.values_per_rating.get(&r).unwrap();

            self.values_per_rating
                .get(&r)
                .unwrap()
                .iter()
                .any(|i| i.0 <= *val && *val < i.1)
        })
    }
}

impl Interval {
    fn apply_condition(&self, cond: &Condition) -> Self {
        let mut result = self.clone();

        match cond.signal {
            '<' => {
                let int_mut = result.values_per_rating.get_mut(&cond.rating).unwrap();
                int_mut.retain(|i| i.0 < cond.number);
                int_mut.iter_mut().for_each(|i| {
                    if i.1 > cond.number {
                        i.1 = cond.number;
                    }
                });
            }
            '>' => {
                let int_mut = result.values_per_rating.get_mut(&cond.rating).unwrap();
                int_mut.retain(|i| i.1 > cond.number);
                int_mut.iter_mut().for_each(|i| {
                    if i.0 < cond.number {
                        i.0 = cond.number + 1;
                    }
                });
            }
            _ => {}
        }

        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Part {
    values_per_rating: BTreeMap<Rating, usize>,
}

impl TryFrom<&str> for Part {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let values = value.strip_prefix("{x=").context("Bad input")?;
        let values = values.strip_suffix("}").context("Bad input")?;
        let (x, values) = values.split_once(",m=").context("Bad input")?;
        let (m, values) = values.split_once(",a=").context("Bad input")?;
        let (a, s) = values.split_once(",s=").context("Bad input")?;

        let x = x.parse::<usize>().context("Bad input")?;
        let m = m.parse::<usize>().context("Bad input")?;
        let a = a.parse::<usize>().context("Bad input")?;
        let s = s.parse::<usize>().context("Bad input")?;

        let mut values_per_rating = BTreeMap::new();

        values_per_rating.insert(Rating::X, x);
        values_per_rating.insert(Rating::M, m);
        values_per_rating.insert(Rating::A, a);
        values_per_rating.insert(Rating::S, s);

        Ok(Self { values_per_rating })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Rule {
    condition: Option<Condition>,
    next_workflow: String,
}

impl TryFrom<&str> for Rule {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value.split_once(":") {
            Some((condition, next_workflow)) => {
                let condition = Some(Condition::try_from(condition)?);
                let next_workflow = next_workflow.to_string();

                Ok(Self {
                    condition,
                    next_workflow,
                })
            }
            None => Ok(Self {
                condition: None,
                next_workflow: value.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Workflow {
    name: String,
    rules: Vec<Rule>,
}

impl TryFrom<&str> for Workflow {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let (name, rules) = value.split_once("{").context("Bad input")?;

        let name = name.to_string();
        let rules = rules.strip_suffix("}").context("Bad input")?;

        let rules = rules
            .split(",")
            .map(|r| Rule::try_from(r))
            .collect::<Result<Vec<Rule>>>()?;

        Ok(Self { name, rules })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct System {
    workflows: BTreeMap<String, Workflow>,
    parts: Vec<Part>,
}

impl TryFrom<&str> for System {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let (workflows, parts) = value.split_once("\n\n").context("Bad input")?;

        let workflows = workflows
            .split("\n")
            .map(|w| Workflow::try_from(w))
            .try_fold(BTreeMap::new(), |mut acc, w| {
                let wo = w?;
                acc.insert(wo.name.clone(), wo);
                anyhow::Ok(acc)
            })?;

        let parts = parts
            .split("\n")
            .map(|p| Part::try_from(p))
            .collect::<Result<Vec<Part>>>()?;

        Ok(Self { workflows, parts })
    }
}

impl System {
    fn find_approved_intervals(&self) -> Vec<Interval> {
        let mut intervals_to_enter_workflow: BTreeMap<&str, Vec<Interval>> = BTreeMap::new();
        let mut pending_workflows = VecDeque::new();

        intervals_to_enter_workflow.insert("in", vec![Interval::all()]);
        pending_workflows.push_back("in");

        while let Some(w) = pending_workflows.pop_front() {
            let workflow = self.workflows.get(w).unwrap();
            let mut remaining_intervals = intervals_to_enter_workflow.get(w).unwrap().clone();

            for r in workflow.rules.iter() {
                match &r.condition {
                    Some(cond) => {
                        intervals_to_enter_workflow
                            .entry(&r.next_workflow)
                            .and_modify(|ex| {
                                remaining_intervals
                                    .iter()
                                    .map(|i| ex.push(i.apply_condition(&cond)))
                                    .count();
                            })
                            .or_insert(
                                remaining_intervals
                                    .iter()
                                    .map(|i| i.apply_condition(&cond))
                                    .collect::<Vec<Interval>>(),
                            );
                        remaining_intervals
                            .iter_mut()
                            .map(|i| *i = i.apply_condition(&cond.invert()))
                            .count();
                    }
                    None => {
                        intervals_to_enter_workflow
                            .entry(&r.next_workflow)
                            .and_modify(|ex| {
                                remaining_intervals
                                    .iter()
                                    .map(|i| ex.push(i.clone()))
                                    .count();
                            })
                            .or_insert(remaining_intervals.clone());
                    }
                }
                if &r.next_workflow != "A" && &r.next_workflow != "R" {
                    pending_workflows.push_back(r.next_workflow.as_str());
                }
            }
        }

        intervals_to_enter_workflow.get("A").unwrap().clone()
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let system = System::try_from(input.trim())?;

    let intervals = system.find_approved_intervals();

    let mut result = 0;

    system.parts.iter().for_each(|p| {
        if intervals.iter().any(|i| i.contains_part(&p)) {
            result += Rating::into_iter()
                .map(|r| p.values_per_rating.get(&r).unwrap())
                .sum::<usize>();
        }
    });

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let system = System::try_from(input.trim())?;

    let intervals = system.find_approved_intervals();

    let mut result = 0;

    intervals.iter().for_each(|i| {
        result += Rating::into_iter()
            .map(|r| {
                i.values_per_rating
                    .get(&r)
                    .unwrap()
                    .iter()
                    .fold(0, |acc, v| acc + v.1 - v.0)
            })
            .product::<usize>();
    });

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d19::run_part_1;
    use crate::d19::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d19/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 19114);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d19/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 418498);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d19/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 167409079868000);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d19/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 123331556462603);
    }
}
