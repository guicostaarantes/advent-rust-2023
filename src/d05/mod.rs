use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MapEntry {
    source: usize,
    length: usize,
    destination: usize,
}

impl TryFrom<&str> for MapEntry {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (destination, source) = value.split_once(" ").context("Bad input")?;
        let (source, length) = source.split_once(" ").context("Bad input")?;
        let source = source.parse::<usize>().context("Bad input")?;
        let length = length.parse::<usize>().context("Bad input")?;
        let destination = destination.parse::<usize>().context("Bad input")?;

        Ok(Self {
            source,
            length,
            destination,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Map {
    entries: Vec<MapEntry>,
}

impl Map {
    fn find_destination(&self, source: usize) -> usize {
        self.entries
            .iter()
            .find(|entry| source >= entry.source && source < entry.source + entry.length)
            .and_then(|entry| Some(source + entry.destination - entry.source))
            .unwrap_or(source)
    }

    fn find_source(&self, destination: usize) -> usize {
        self.entries
            .iter()
            .find(|entry| {
                destination >= entry.destination && destination < entry.destination + entry.length
            })
            .and_then(|entry| Some(destination + entry.source - entry.destination))
            .unwrap_or(destination)
    }

    fn sort_entries_by_source(&mut self) {
        self.entries
            .sort_by(|a, b| a.source.partial_cmp(&b.source).unwrap());
    }

    fn sort_entries_by_destination(&mut self) {
        self.entries
            .sort_by(|a, b| a.destination.partial_cmp(&b.destination).unwrap());
    }

    fn merge(&self, rhs: &Map) -> Map {
        let mut ranges = Vec::new();
        self.entries.iter().for_each(|entry| {
            let start = entry.source;
            let end = entry.source + entry.length;
            if !ranges.contains(&start) {
                ranges.push(start);
            }
            if !ranges.contains(&end) {
                ranges.push(end);
            }
        });
        rhs.entries.iter().for_each(|entry| {
            let start = self.find_source(entry.source);
            let end = self.find_source(entry.source + entry.length);
            if !ranges.contains(&start) {
                ranges.push(start);
            }
            if !ranges.contains(&end) {
                ranges.push(end);
            }
        });
        ranges.sort_by(|a, b| a.partial_cmp(&b).unwrap());

        let mut entries = Vec::new();
        for k in 0..ranges.len() - 1 {
            let source = ranges[k];
            let next_source = ranges[k + 1];
            let final_destination = rhs.find_destination(self.find_destination(source));
            entries.push(MapEntry {
                source,
                length: next_source - source,
                destination: final_destination,
            });
        }

        Map { entries }
    }
}

impl TryFrom<&str> for Map {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let entries = value
            .lines()
            .map(|line| MapEntry::try_from(line))
            .collect::<Result<Vec<MapEntry>>>()?;

        Ok(Self { entries })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Almanac {
    seeds: Vec<usize>,
    maps: Vec<Map>,
}

impl TryFrom<&str> for Almanac {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let (seeds, maps) = value.split_once("\n\n").context("Bad input")?;

        let (_, seeds) = seeds.split_once(": ").context("Bad input")?;
        let seeds = seeds
            .split(" ")
            .map(|seed| seed.parse::<usize>().context("Bad input"))
            .collect::<Result<Vec<usize>>>()?;

        let maps = maps
            .split("\n\n")
            .map(|map| {
                let (_, content) = map.split_once("\n").context("Bad input")?;
                Map::try_from(content)
            })
            .collect::<Result<Vec<Map>>>()?;

        Ok(Self { seeds, maps })
    }
}

impl Almanac {
    fn optimal_map(&self) -> Map {
        let mut optimal_map = self.maps[0].clone();

        for k in 1..self.maps.len() {
            optimal_map = optimal_map.merge(&self.maps[k]);
        }

        optimal_map.sort_entries_by_source();

        for k in 1..optimal_map.entries.len() {
            let current = optimal_map.entries.get(k).unwrap();
            let previous = optimal_map.entries.get(k - 1).unwrap();
            let previous_end = previous.source + previous.length;
            if current.source > previous_end {
                optimal_map.entries.push(MapEntry {
                    source: previous_end,
                    length: current.source - previous_end,
                    destination: previous_end,
                })
            }
        }

        optimal_map.sort_entries_by_destination();

        optimal_map
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let almanac = Almanac::try_from(input.trim())?;

    let optimal_map = almanac.optimal_map();

    let result = almanac
        .seeds
        .iter()
        .map(|seed| optimal_map.find_destination(*seed))
        .min()
        .context("Bad min")?;

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let almanac = Almanac::try_from(input.trim())?;
    let optimal_map = almanac.optimal_map();

    let mut seed_ranges = Vec::new();
    for k in 0..almanac.seeds.len() / 2 {
        seed_ranges.push((
            almanac.seeds[2 * k],
            almanac.seeds[2 * k] + almanac.seeds[2 * k + 1] - 1,
        ));
    }
    seed_ranges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut k = 0;
    let best_seed = 'res: loop {
        if let Some(entry) = optimal_map.entries.get(k) {
            for seed_range in seed_ranges.iter() {
                let entry_end = entry.source + entry.length - 1;
                if !(seed_range.0 > entry_end || seed_range.1 < entry.source) {
                    break 'res if seed_range.0 > entry.source {
                        seed_range.0
                    } else {
                        entry.source
                    };
                }
            }
        } else {
            // trivial case: no seeds match any maps, smallest seed wins
            break 'res seed_ranges[0].0;
        }
        k += 1;
    };

    let result = optimal_map.find_destination(best_seed);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d05::run_part_1;
    use crate::d05::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d05/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 35);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d05/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 174137457);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d05/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 46);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d05/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 1493866);
    }
}
