use std::collections::{HashMap, hash_map, HashSet};

use rand::{prelude::ThreadRng, Rng};

use crate::game::{Game, Card, self};

pub struct ClientSession {
    id: usize,
    username: String,
    is_spymaster: bool
}

#[derive(Debug, Clone)]
pub struct Room {
    name: String,
    game_id: usize,
    sessions: Vec<usize>
}

impl Room {
    fn new(name: String, game_id: usize) -> Room {
        Room { name, game_id, sessions: Vec::new() }
    }
}

pub trait Database {
    fn create_room(&mut self, name: &String) -> Room;
    fn remove_room(&mut self, name: &String) -> Room;
    fn get_room(&self, name: &String) -> Room;
    fn add_session_to_room(&mut self, session: ClientSession, room: String);
    fn remove_session_from_room(&mut self, session_id: usize);
    fn update_game(&mut self, game_id: usize, game_update: Game) -> Game;
    fn get_game(&self, game_id: usize) -> Game;
}

pub struct MemoryDatabase {
    rng: ThreadRng,
    rooms: HashMap<String, Room>,
    games: HashMap<usize, Game>,
    sessions: HashMap<usize, ClientSession>
}

impl MemoryDatabase {
    fn new() -> MemoryDatabase {
        MemoryDatabase {
            rng: rand::thread_rng(),
            rooms: HashMap::new(),
            games: HashMap::new(),
            sessions: HashMap::new()
        }
    }
}

impl Database for MemoryDatabase {
    fn get_room(&self, name: &String) -> Room {
        self.rooms.get(name).expect("Could not find room").clone()
    }

    fn create_room(&mut self, name: &String) -> Room {
        match self.rooms.entry(name.to_string()) {
            hash_map::Entry::Occupied(room) => {
                panic!("Room {} already exists!", room.get().name)
            },
            hash_map::Entry::Vacant(_) => {
                let game_id = self.rng.gen::<usize>();
                self.games.insert(game_id, Game::new());
                let new_room = Room::new(name.clone(), game_id);
                self.rooms.insert(name.clone(), new_room.clone());
                new_room
            }
        }
    }

    fn remove_room(&mut self, name: &String) -> Room {
        todo!()
    }

    fn add_session_to_room(&mut self, session: ClientSession, room: String) {
        todo!()
    }

    fn remove_session_from_room(&mut self, session_id: usize) {
        todo!()
    }

    fn update_game(&mut self, game_id: usize, game_update: Game) -> Game {
        todo!()
    }

    fn get_game(&self, game_id: usize) -> Game {
        self.games.get(&game_id).expect(&format!("Could not find game with id {}", game_id)).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{Database, MemoryDatabase};

    #[test]
    fn creates_room() {
        let mut db = MemoryDatabase::new();
        let room_name = &String::from("foo");
        db.create_room(&String::from("foo"));
        let room = db.get_room(&room_name);
        println!("{:?}", &room);
        println!("{:?}", db.get_game(room.game_id))
    }
}