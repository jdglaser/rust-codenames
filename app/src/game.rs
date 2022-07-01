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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Team {
    RED,
    BLUE,
}

impl Team {
    fn opposite(team: &Team) -> Team {
        match team {
            Team::RED => Team::BLUE,
            Team::BLUE => Team::RED,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    word: String,
    card_type: CardType,
    flipped: bool,
    coord: (usize, usize),
}

impl Card {
    fn new(word: String, card_type: CardType, coord: (usize, usize)) -> Card {
        Card {
            word,
            card_type,
            flipped: false,
            coord,
        }
    }

    pub fn get_card_type(&self) -> CardType {
        self.card_type.clone()
    }
}

impl Card {
    pub fn flip(&mut self) {
        self.flipped = true;
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
    sessions: HashSet<usize>,
    starting_team: Team,
    turn_team: Team,
    board: Board,
}

impl Game {
    pub fn new() -> Self {
        let starting_team = Team::BLUE;
        return Game {
            board: Game::create_board(&starting_team),
            turn_team: starting_team.clone(),
            starting_team,
            sessions: HashSet::new(),
        };
    }

    pub fn new_from_game(game: &Game) -> Self {
        let starting_team = Team::opposite(&game.starting_team);
        return Game {
            board: Game::create_board(&starting_team),
            turn_team: starting_team.clone(),
            starting_team,
            sessions: game.sessions.clone(),
        };
    }

    pub fn new_from_current_game(&self) -> Self {
        Game::new_from_game(self)
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

    pub fn add_player(&mut self, id: usize) -> &Self {
        self.sessions.insert(id);
        self
    }

    pub fn remove_player(&mut self, id: &usize) {
        self.sessions.remove(id);
    }

    pub fn get_sessions(&self) -> &HashSet<usize> {
        &self.sessions
    }

    pub fn get_board(&self) -> &Board {
        &self.board
    }

    pub fn flip_card(&mut self, coord: (usize, usize)) -> Card {
        let card = &mut self.board[coord.0][coord.1];
        if card.card_type == CardType::ASSASSIN {
            todo!("Implement game over");
        }
        if card.card_type != CardType::from_team(&self.turn_team) {
            card.flip();
        }
        self.turn_team = Team::opposite(&self.turn_team);
        card.clone()
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
