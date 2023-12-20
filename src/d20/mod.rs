use std::collections::{BTreeMap, VecDeque};

use anyhow::{Context, Result};

fn greatest_common_divisor(a: usize, b: usize) -> usize {
    if b == 0 {
        return a;
    }
    greatest_common_divisor(b, a % b)
}

fn least_common_multiple(nums: &[usize]) -> usize {
    if nums.len() == 1 {
        return nums[0];
    }
    let a = nums[0];
    let b = least_common_multiple(&nums[1..]);
    a * b / greatest_common_divisor(a, b)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum OnOff {
    On,
    Off,
}

impl OnOff {
    fn flip(&mut self) {
        *self = match self {
            OnOff::On => OnOff::Off,
            OnOff::Off => OnOff::On,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Pulse {
    Low,
    High,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PendingPulse {
    from: String,
    to: String,
    pulse: Pulse,
}

impl std::fmt::Debug for PendingPulse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -{:?}-> {}", self.from, self.pulse, self.to)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BroadcasterModule {
    name: String,
    destination_modules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct FlipFlopModule {
    name: String,
    destination_modules: Vec<String>,
    current_state: OnOff,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ConjunctionModule {
    name: String,
    destination_modules: Vec<String>,
    current_state: BTreeMap<String, Pulse>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Module {
    Broadcaster(BroadcasterModule),
    FlipFlop(FlipFlopModule),
    Conjunction(ConjunctionModule),
}

impl TryFrom<&str> for Module {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let (name, modules) = value.split_once(" -> ").context("Bad input")?;

        if name == "broadcaster" {
            let modules = modules
                .split(", ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            Ok(Self::Broadcaster(BroadcasterModule {
                name: name.to_string(),
                destination_modules: modules,
            }))
        } else if let Some((_, name)) = name.split_once("%") {
            let modules = modules
                .split(", ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            Ok(Self::FlipFlop(FlipFlopModule {
                name: name.to_string(),
                destination_modules: modules,
                current_state: OnOff::Off,
            }))
        } else if let Some((_, name)) = name.split_once("&") {
            let modules = modules
                .split(", ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            Ok(Self::Conjunction(ConjunctionModule {
                name: name.to_string(),
                destination_modules: modules,
                current_state: BTreeMap::new(),
            }))
        } else {
            Err(anyhow::anyhow!("Unknown module"))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Program {
    modules: BTreeMap<String, Module>,
    pending_pulses: VecDeque<PendingPulse>,
    low_pulse_count: usize,
    high_pulse_count: usize,
}

impl TryFrom<&str> for Program {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut modules = BTreeMap::new();

        value
            .lines()
            .map(|line| {
                let module = Module::try_from(line)?;
                match module {
                    Module::Broadcaster(ref mo) => {
                        modules.insert(mo.name.clone(), module);
                    }
                    Module::FlipFlop(ref mo) => {
                        modules.insert(mo.name.clone(), module);
                    }
                    Module::Conjunction(ref mo) => {
                        modules.insert(mo.name.clone(), module);
                    }
                };
                anyhow::Ok(())
            })
            .count();

        {
            let modules_copy = modules.clone();
            modules.iter_mut().for_each(|mo| match mo.1 {
                Module::Conjunction(m) => {
                    modules_copy.values().for_each(|v| match v {
                        Module::Broadcaster(m2) => {
                            if m2.destination_modules.contains(&m.name) {
                                m.current_state.insert(m2.name.clone(), Pulse::Low);
                            }
                        }
                        Module::FlipFlop(m2) => {
                            if m2.destination_modules.contains(&m.name) {
                                m.current_state.insert(m2.name.clone(), Pulse::Low);
                            }
                        }
                        Module::Conjunction(m2) => {
                            if m2.destination_modules.contains(&m.name) {
                                m.current_state.insert(m2.name.clone(), Pulse::Low);
                            }
                        }
                    });
                }
                _ => {}
            });
        }

        Ok(Self {
            modules,
            pending_pulses: VecDeque::new(),
            low_pulse_count: 0,
            high_pulse_count: 0,
        })
    }
}

impl Program {
    fn emit_pulse(&mut self, pending_pulse: PendingPulse) {
        match pending_pulse.pulse {
            Pulse::Low => self.low_pulse_count += 1,
            Pulse::High => self.high_pulse_count += 1,
        };
        self.pending_pulses.push_back(pending_pulse);
    }
}

impl Program {
    fn press_button(&mut self) {
        self.emit_pulse(PendingPulse {
            from: "button".to_string(),
            to: "broadcaster".to_string(),
            pulse: Pulse::Low,
        });
    }
}

impl Program {
    fn process_next_pulse(&mut self) -> Option<PendingPulse> {
        let mut pulses_to_emit = Vec::new();

        if let Some(pending_pulse) = self.pending_pulses.pop_front() {
            if let Some(module) = self.modules.get_mut(&pending_pulse.to) {
                match module {
                    Module::Broadcaster(mo) => {
                        mo.destination_modules.iter().for_each(|to| {
                            pulses_to_emit.push(PendingPulse {
                                from: mo.name.clone(),
                                to: to.clone(),
                                pulse: pending_pulse.pulse.clone(),
                            });
                        });
                    }
                    Module::FlipFlop(mo) => {
                        if let Pulse::Low = pending_pulse.pulse {
                            // emits pulses if receives a low pulse
                            mo.destination_modules.iter().for_each(|to| {
                                pulses_to_emit.push(PendingPulse {
                                    from: mo.name.clone(),
                                    to: to.clone(),
                                    pulse: match mo.current_state {
                                        OnOff::On => Pulse::Low,
                                        OnOff::Off => Pulse::High,
                                    },
                                });
                            });
                            // flips if receives a low pulse
                            mo.current_state.flip();
                        }
                    }
                    Module::Conjunction(mo) => {
                        if let Some(mem) = mo.current_state.get_mut(&pending_pulse.from) {
                            *mem = pending_pulse.pulse.clone();
                        }
                        mo.destination_modules.iter().for_each(|to| {
                            pulses_to_emit.push(PendingPulse {
                                from: mo.name.clone(),
                                to: to.clone(),
                                pulse: if mo.current_state.values().all(|v| *v == Pulse::High) {
                                    Pulse::Low
                                } else {
                                    Pulse::High
                                },
                            });
                        });
                    }
                }
            }

            pulses_to_emit.into_iter().for_each(|p| self.emit_pulse(p));

            // return the pending pulse
            Some(pending_pulse)
        } else {
            // return None to let the consumer know that the pending pulses queue is empty
            None
        }
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let mut program = Program::try_from(input.trim())?;

    for _ in 0..1000 {
        program.press_button();
        loop {
            if let None = program.process_next_pulse() {
                break;
            }
        }
    }

    Ok(program.low_pulse_count * program.high_pulse_count)
}

/**
 * This solution should work for every input, but it can take a long time.
 *
 * For the given case, it would have taken 75 years at a rate of 100_000 button presses per
 * second.
 */
pub fn run_part_2_general_but_slow(input: String) -> Result<usize> {
    let mut program = Program::try_from(input.trim())?;

    let mut button_presses = 0;

    'res: loop {
        if button_presses == 10_000 {
            return Err(anyhow::anyhow!("Too many steps to brute force"));
        }

        program.press_button();
        button_presses += 1;
        loop {
            if let Some(pp) = program.process_next_pulse() {
                if pp.pulse == Pulse::Low && pp.from == "rx".to_string() {
                    break 'res;
                }
            } else {
                break;
            }
        }
    }

    Ok(button_presses)
}

/**
 * By studying the input, we can see that rx is attached to a single conjunction (in this case ll)
 * which is attached to four conjunctions (in this case vb, kl, kv, vm). For ll to send a low pulse
 * to rx, it has to have received high pulse (latest) from the four conjunctions of the second layer.
 *
 * By logging some of the results, we also notice that this second layer of conjunctions was carefully
 * made to send a high pulse and an immediate low pulse within the same button press after a
 * specific number of button presses.
 *
 * By assuming the above, it's possible to break the problem into four (should also work for other
 * numbers) subproblems, and the answer should be the LCM of the subanswers.
 *
 * For this case, the subanswers were four prime numbers in the range of 3700-4100, specifically
 * picked to make the LCM a huge number (hence making brute force unfeasible).
 */
pub fn run_part_2(input: String) -> Result<usize> {
    let mut program = Program::try_from(input.trim())?;

    let mut button_presses = 0;
    let mut result = 1;

    let mut rx_conjunctions = program.modules.values().filter(|v| match v {
        Module::Broadcaster(b) => b.destination_modules.contains(&"rx".to_string()),
        Module::FlipFlop(f) => f.destination_modules.contains(&"rx".to_string()),
        Module::Conjunction(c) => c.destination_modules.contains(&"rx".to_string()),
    });

    let first_layer_conjunction = if let Some(Module::Conjunction(c)) = rx_conjunctions.next() {
        Ok(c.name.clone())
    } else {
        Err(anyhow::anyhow!("Invalid input for this optimized function"))
    }?;
    if let Some(_) = rx_conjunctions.next() {
        return Err(anyhow::anyhow!("Invalid input for this optimized function"));
    }

    let mut second_layer_conjunctions = program
        .modules
        .values()
        .filter(|v| match v {
            Module::Broadcaster(b) => b.destination_modules.contains(&first_layer_conjunction),
            Module::FlipFlop(f) => f.destination_modules.contains(&first_layer_conjunction),
            Module::Conjunction(c) => c.destination_modules.contains(&first_layer_conjunction),
        })
        .map(|v| match v {
            Module::Broadcaster(_) => {
                Err(anyhow::anyhow!("Invalid input for this optimized function"))
            }
            Module::FlipFlop(_) => {
                Err(anyhow::anyhow!("Invalid input for this optimized function"))
            }
            Module::Conjunction(c) => Ok(c.name.clone()),
        })
        .collect::<Result<Vec<String>>>()?;

    dbg!(&second_layer_conjunctions);

    let mut high_pulse_in_this_button_press = Vec::new();

    while second_layer_conjunctions.len() > 0 {
        high_pulse_in_this_button_press.clear();
        button_presses += 1;
        program.press_button();
        loop {
            if let Some(pp) = program.process_next_pulse() {
                if pp.pulse == Pulse::High && second_layer_conjunctions.contains(&pp.from) {
                    high_pulse_in_this_button_press.push(pp.from.clone());
                } else if pp.pulse == Pulse::Low && high_pulse_in_this_button_press.contains(&pp.from) {
                    result = least_common_multiple(&[result, button_presses]);
                    high_pulse_in_this_button_press.retain(|c| *c != pp.from);
                    second_layer_conjunctions.retain(|c| *c != pp.from);
                }
            } else {
                break;
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d20::run_part_1;
    use crate::d20::run_part_2;
    use crate::d20::run_part_2_general_but_slow;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d20/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 11687500);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d20/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 743090292);
    }

    #[test]
    fn part_2_brute_force_prod() {
        let input = read_to_string("src/d20/prod.txt").expect("could not read file");
        assert_eq!(
            format!("{}", run_part_2_general_but_slow(input).unwrap_err()),
            "Too many steps to brute force",
        );
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d20/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 241528184647003);
    }
}
