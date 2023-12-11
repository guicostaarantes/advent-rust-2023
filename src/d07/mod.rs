use std::cmp::Ordering;
use std::collections::BTreeMap;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Card {
    Joker,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl TryFrom<char> for Card {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            '2' => Ok(Card::Two),
            '3' => Ok(Card::Three),
            '4' => Ok(Card::Four),
            '5' => Ok(Card::Five),
            '6' => Ok(Card::Six),
            '7' => Ok(Card::Seven),
            '8' => Ok(Card::Eight),
            '9' => Ok(Card::Nine),
            'T' => Ok(Card::Ten),
            'J' => Ok(Card::Jack),
            'Q' => Ok(Card::Queen),
            'K' => Ok(Card::King),
            'A' => Ok(Card::Ace),
            _ => Err(anyhow::anyhow!("Invalid card")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum HandKind {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord)]
struct Hand {
    cards: Vec<Card>,
}

impl TryFrom<&str> for Hand {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let cards = value
            .chars()
            .map(|c| Card::try_from(c))
            .collect::<Result<Vec<Card>>>()
            .context("At least one card is invalid")?;

        if cards.len() != 5 {
            return Err(anyhow::anyhow!("Hands need 5 cards exactly"));
        }

        Ok(Self { cards })
    }
}

impl Hand {
    fn count_cards(&self) -> BTreeMap<Card, usize> {
        let mut result = BTreeMap::new();
        for k in self.cards.iter() {
            result.entry(k.clone()).and_modify(|v| *v += 1).or_insert(1);
        }
        result
    }

    fn get_kind(&self) -> HandKind {
        let mut values = self
            .count_cards()
            .iter()
            .filter(|(v, _)| v != &&Card::Joker)
            .map(|(_, k)| k.clone())
            .collect::<Vec<usize>>();

        values.sort_by(|a, b| b.partial_cmp(&a).unwrap());

        match values[..] {
            // Five jokers
            [] => HandKind::FiveOfAKind,
            // Four jokers
            [1] => HandKind::FiveOfAKind,
            // Three jokers
            [1, 1] => HandKind::FourOfAKind,
            [2] => HandKind::FiveOfAKind,
            // Two jokers
            [1, 1, 1] => HandKind::ThreeOfAKind,
            [2, 1] => HandKind::FourOfAKind,
            [3] => HandKind::FiveOfAKind,
            // One joker
            [1, 1, 1, 1] => HandKind::OnePair,
            [2, 1, 1] => HandKind::ThreeOfAKind,
            [2, 2] => HandKind::FullHouse,
            [3, 1] => HandKind::FourOfAKind,
            [4] => HandKind::FiveOfAKind,
            // No jokers
            [1, 1, 1, 1, 1] => HandKind::HighCard,
            [2, 1, 1, 1] => HandKind::OnePair,
            [2, 2, 1] => HandKind::TwoPair,
            [3, 1, 1] => HandKind::ThreeOfAKind,
            [3, 2] => HandKind::FullHouse,
            [4, 1] => HandKind::FourOfAKind,
            [5] => HandKind::FiveOfAKind,
            _ => unreachable!(),
        }
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.get_kind().partial_cmp(&other.get_kind()) {
            Some(Ordering::Less) => Some(Ordering::Less),
            Some(Ordering::Greater) => Some(Ordering::Greater),
            Some(Ordering::Equal) => {
                let mut k = 0;
                loop {
                    match self.cards[k].partial_cmp(&other.cards[k]) {
                        Some(Ordering::Less) => {
                            break Some(Ordering::Less);
                        }
                        Some(Ordering::Greater) => {
                            break Some(Ordering::Greater);
                        }
                        Some(Ordering::Equal) => {
                            k += 1;
                            if k == 5 {
                                break Some(Ordering::Equal);
                            }
                        }
                        None => break None,
                    }
                }
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Player {
    hand: Hand,
    bid: usize,
}

impl TryFrom<&str> for Player {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let (hand, bid) = value.split_once(" ").context("Bad input, no space found")?;
        let hand = Hand::try_from(hand)?;
        let bid = bid.parse::<usize>().context("Bid is not a number")?;

        Ok(Self { hand, bid })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Round {
    players: Vec<Player>,
}

impl TryFrom<&str> for Round {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let players = value
            .split("\n")
            .map(|player| Player::try_from(player))
            .collect::<Result<Vec<Player>>>()?;

        Ok(Self { players })
    }
}

impl Round {
    fn set_jokers(mut self) -> Self {
        self.players.iter_mut().for_each(|pl| {
            pl.hand.cards.iter_mut().for_each(|card| {
                if card == &mut Card::Jack {
                    *card = Card::Joker
                };
            });
        });

        self
    }
}

pub fn run_part_1(input: String) -> Result<usize> {
    let mut result = 0;

    let mut round = Round::try_from(input.trim())?;

    round
        .players
        .sort_by(|a, b| a.hand.partial_cmp(&b.hand).unwrap());

    round
        .players
        .iter()
        .enumerate()
        .for_each(|(k, pl)| result += pl.bid * (k + 1));

    Ok(result)
}

pub fn run_part_2(input: String) -> Result<usize> {
    let mut result = 0;

    let mut round = Round::try_from(input.trim())?.set_jokers();

    round
        .players
        .sort_by(|a, b| a.hand.partial_cmp(&b.hand).unwrap());

    round
        .players
        .iter()
        .enumerate()
        .for_each(|(k, pl)| result += pl.bid * (k + 1));

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::d07::run_part_1;
    use crate::d07::run_part_2;
    use std::fs::read_to_string;

    #[test]
    fn part_1_test() {
        let input = read_to_string("src/d07/test.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 6440);
    }

    #[test]
    fn part_1_prod() {
        let input = read_to_string("src/d07/prod.txt").expect("could not read file");
        assert_eq!(run_part_1(input).unwrap(), 246163188);
    }

    #[test]
    fn part_2_test() {
        let input = read_to_string("src/d07/test.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 5905);
    }

    #[test]
    fn part_2_prod() {
        let input = read_to_string("src/d07/prod.txt").expect("could not read file");
        assert_eq!(run_part_2(input).unwrap(), 245794069);
    }
}
