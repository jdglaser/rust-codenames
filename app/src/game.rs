use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

use log::{debug, info};
use rand::{prelude::SliceRandom, Rng};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum CardType {
    RED,
    BLUE,
    BYSTANDER,
    ASSASSIN,
}

impl CardType {
    fn from_team(team: &Team) -> CardType {
        match team {
            Team::BLUE => CardType::BLUE,
            Team::RED => CardType::RED,
            _ => panic!(
                "Cannot create a CardType enum variant from Team variant: '{:?}'",
                team
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Team {
    RED,
    BLUE,
}

impl Team {
    pub fn opposite(team: &Team) -> Team {
        match team {
            Team::RED => Team::BLUE,
            Team::BLUE => Team::RED,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub word: String,
    pub card_type: CardType,
    pub flipped: bool,
    pub coord: (usize, usize)
}

impl Card {
    fn new(word: String, card_type: CardType, coord: (usize, usize)) -> Card {
        Card {
            word,
            card_type,
            flipped: false,
            coord
        }
    }
}

impl Default for Card {
    fn default() -> Self {
        Card::new(String::from(""), CardType::BYSTANDER, (0, 0))
    }
}

pub type Board = [[Card; 5]; 5];

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub starting_team: Team,
    pub turn_team: Team,
    pub board: Board,
    pub remaining_cards: (u8, u8),
    pub game_over: bool
}

impl Game {
    pub fn new() -> Self {
        let starting_team = Team::BLUE;
        return Game {
            board: Game::create_board(&starting_team),
            turn_team: starting_team.clone(),
            remaining_cards: Game::initalize_remaining_cards(&starting_team),
            starting_team,
            game_over: false
        };
    }

    pub fn new_from_game(game: &Game) -> Self {
        let starting_team = Team::opposite(&game.starting_team);
        return Game {
            board: Game::create_board(&starting_team),
            turn_team: starting_team.clone(),
            remaining_cards: Game::initalize_remaining_cards(&starting_team),
            starting_team,
            game_over: false
        };
    }

    pub fn new_from_current_game(&self) -> Self {
        Game::new_from_game(self)
    }

    fn initalize_remaining_cards(starting_team: &Team) -> (u8, u8) {
        match starting_team {
            Team::BLUE => (9, 8),
            Team::RED => (8, 9)
        }
    }

    fn fill_card(board: &mut Board, card_type: &CardType) {
        let mut rng = rand::thread_rng();
        let mut card: &mut Card;
        card = &mut board[rng.gen_range(0..5)][rng.gen_range(0..5)];
        loop {
            match card.card_type {
                CardType::BYSTANDER => {
                    card.card_type = card_type.clone();
                    break;
                }
                _ => {
                    debug!("Card already taken trying again");
                    card = &mut board[rng.gen_range(0..5)][rng.gen_range(0..5)];
                }
            }
        }
    }

    fn create_board(starting_team: &Team) -> Board {
        let words = Game::get_words();

        let mut board: Board = Default::default();
        for row in 0..5 {
            for col in 0..5 {
                let random_word = words.choose(&mut rand::thread_rng()).unwrap().clone();
                board[row][col] = Card::new(random_word, CardType::BYSTANDER, (row, col));
            }
        }

        for i in 0..9 {
            // Assign starting team
            Game::fill_card(&mut board, &CardType::from_team(starting_team));

            // Assign opposite team (second team has -1 card)
            if i != 8 {
                Game::fill_card(
                    &mut board,
                    &CardType::from_team(&Team::opposite(starting_team)),
                )
            }
        }

        // Assign assassin
        Game::fill_card(&mut board, &CardType::ASSASSIN);
        board
    }

    fn get_words() -> Vec<String> {
        let file = File::open("./src/words.txt").expect("Unable to open file");
        let buf = BufReader::new(file);
        buf.lines()
            .map(|l| l.expect("Could not parse line from file"))
            .collect()
    }

    pub fn flip_card(&self, coord: (usize, usize)) -> Game {
        let mut new_game = self.clone();
        let card = &mut new_game.board[coord.0][coord.1];

        match card.card_type {
            CardType::BLUE => {
                new_game.remaining_cards.0 -= 1;
                if let Team::RED = new_game.turn_team {
                    new_game.turn_team = Team::opposite(&new_game.turn_team);
                }
            },
            CardType::RED => {
                new_game.remaining_cards.1 -= 1;
                if let Team::BLUE = new_game.turn_team {
                    new_game.turn_team = Team::opposite(&new_game.turn_team);
                }
            },
            _ => new_game.turn_team = Team::opposite(&new_game.turn_team)
        };

        if new_game.remaining_cards.0 == 0 || new_game.remaining_cards.1 == 0 {
            new_game.game_over = true;
        }

        *card = Card {
            flipped: true, 
            ..card.clone()
        };

        new_game
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, Card, CardType, Game};

    fn find_cards_in_board(board: &Board, card_type: &CardType) -> Vec<Card> {
        let mut cards: Vec<Card> = Vec::new();
        for row in 0..5 {
            for col in 0..5 {
                if *card_type == board[row][col].card_type {
                    cards.push(board[row][col].clone());
                }
            }
        }
        cards
    }

    #[test]
    fn fills_card() {
        let mut board: Board = Default::default();
        for _ in 0..3 {
            Game::fill_card(&mut board, &CardType::BLUE);
        }

        for _ in 0..2 {
            Game::fill_card(&mut board, &CardType::RED);
        }

        Game::fill_card(&mut board, &CardType::ASSASSIN);

        assert_eq!(find_cards_in_board(&board, &CardType::BLUE).len(), 3);
        assert_eq!(find_cards_in_board(&board, &CardType::RED).len(), 2);
        assert_eq!(find_cards_in_board(&board, &CardType::ASSASSIN).len(), 1);
    }

    #[test]
    fn creates_new_board() {
        let mut game = Game::new();
        assert_eq!(find_cards_in_board(&game.board, &CardType::BLUE).len(), 9);
        assert_eq!(find_cards_in_board(&game.board, &CardType::RED).len(), 8);
        assert_eq!(
            find_cards_in_board(&game.board, &CardType::ASSASSIN).len(),
            1
        );

        game = Game::new_from_game(&game);
        assert_eq!(find_cards_in_board(&game.board, &CardType::BLUE).len(), 8);
        assert_eq!(find_cards_in_board(&game.board, &CardType::RED).len(), 9);
        assert_eq!(
            find_cards_in_board(&game.board, &CardType::ASSASSIN).len(),
            1
        );
    }
}
