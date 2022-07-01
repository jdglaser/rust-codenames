use std::{collections::{hash_map, HashMap}};

use rand::{Rng};
use anyhow::{Result, bail, Context, anyhow};

use crate::game::{Game};

#[derive(Clone)]
pub struct ClientSession {
    id: usize,
    username: String,
    is_spymaster: bool,
}

impl ClientSession {
    fn new() -> ClientSession {
        let id = rand::thread_rng().gen(); 
        ClientSession {
            id,
            username: String::from(""),
            is_spymaster: false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Room {
    name: String,
    game_id: usize,
    sessions: Vec<usize>,
}

impl Room {
    fn new(name: String, game_id: usize) -> Room {
        Room {
            name,
            game_id,
            sessions: Vec::new(),
        }
    }
}


pub trait Database {
    fn create_room(&mut self, name: &String) -> Result<Room>;
    fn remove_room(&mut self, name: &String) -> Result<Room>;
    fn get_room(&self, name: &String) -> Result<&Room>;
    fn add_session_to_room(&mut self, session: &ClientSession, room: &String) -> Result<()>;
    fn remove_session_from_room(&mut self, session_id: usize) -> Result<()>;
    fn update_game(&mut self, game_id: usize, game_update: &Game) -> Result<Game>;
    fn get_game(&self, game_id: usize) -> Result<&Game>;
}

#[derive(Clone)]
pub struct MemoryDatabase {
    rooms: HashMap<String, Room>,
    games: HashMap<usize, Game>,
    sessions: HashMap<usize, ClientSession>,
}

impl MemoryDatabase {
    pub fn new() -> MemoryDatabase {
        MemoryDatabase {
            rooms: HashMap::new(),
            games: HashMap::new(),
            sessions: HashMap::new(),
        }
    }
}

impl Database for MemoryDatabase {
    fn get_room(&self, name: &String) -> Result<&Room> {
        self.rooms
            .get(name)
            .context(format!("Could not find room with name '{}'.", name))
    }

    fn create_room(&mut self, name: &String) -> Result<Room> {
        match self.rooms.entry(name.to_string()) {
            hash_map::Entry::Occupied(room) => {
                bail!("Room {} already exists!", room.get().name)
            }
            hash_map::Entry::Vacant(_) => {
                let game_id = rand::thread_rng().gen();
                self.games.insert(game_id, Game::new());
                let new_room = Room::new(name.clone(), game_id);
                self.rooms.insert(name.clone(), new_room.clone());
                Ok(new_room)
            }
        }
    }

    fn remove_room(&mut self, name: &String) -> Result<Room> {
        self.rooms
            .remove(name)
            .context(format!("Failed to remove room with name '{}' because it did not exist.", name))
    }

    fn add_session_to_room(&mut self, session: &ClientSession, room: &String) -> Result<()> {
        match self.rooms.get_mut(room) {
            Some(val) => {
                val.sessions.push(session.id);
                Ok(())
            },
            None => Err(anyhow!("Could not find room with name '{}'.", room))
        }
    }

    fn remove_session_from_room(&mut self, session_id: usize) -> Result<()> {
        todo!()
    }

    fn update_game(&mut self, game_id: usize, game_update: &Game) -> Result<Game> {
        todo!()
    }

    fn get_game(&self, game_id: usize) -> Result<&Game> {
        self.games
            .get(&game_id)
            .context(format!("Could not find game with id '{}'.", game_id))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::error;

    use super::{Database, MemoryDatabase, ClientSession};

    #[test]
    fn creates_gets_removes_room() {
        let mut db = MemoryDatabase::new();
        let room_name = String::from("foo");
        db.create_room(&String::from("foo")).unwrap();
        let room = db.get_room(&room_name).unwrap();
        
        assert_eq!(room_name, room.name);
        assert_eq!(0, room.sessions.len());
        
        let game = db.get_game(room.game_id).unwrap();
        assert_eq!(5, game.get_board().len());

        let removed_room = db.remove_room(&room_name).unwrap();
        assert_eq!(room_name, removed_room.name);

        let bad_get = db.get_room(&room_name);
        assert!(bad_get.is_err());
        let error_msg = bad_get.expect_err("Did not get an error").to_string();
        assert_eq!("Could not find room with name 'foo'.", error_msg);
    }

    #[test]
    fn adds_and_removes_session_from_room() {
        let mut db = MemoryDatabase::new();
        let room_name = String::from("foo");
        let new_session = ClientSession::new();

        // Add to room that doesn't exist
        let bad_result = db.add_session_to_room(&new_session, &room_name);
        assert!(bad_result.is_err());
        let error_msg = bad_result.expect_err("Did not get an error").to_string();
        assert_eq!("Could not find room with name 'foo'.", error_msg)
        
        // db.create_room(&String::from("foo")).unwrap();
    }
}
