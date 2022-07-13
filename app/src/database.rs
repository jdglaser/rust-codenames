use std::{collections::{HashMap}, sync::{Mutex, Arc, MutexGuard}};

use rand::{Rng};
use anyhow::{Result, bail, Context, anyhow};

use crate::{game::{Game}, client::ClientSession, server::Room};

pub trait Database {
    fn create_room(&mut self, name: &String) -> Result<String>;
    fn remove_room(&mut self, name: &String) -> Result<()>;
    fn get_room(&self, name: &String) -> Result<Room>;
    fn get_rooms(&self) -> Result<Vec<Room>>;
    fn get_sessions(&self) -> Result<Vec<ClientSession>>;
    fn get_session(&self, id: &usize) -> Result<ClientSession>;
    fn update_session(&mut self, id: usize, session_update: &ClientSession) -> Result<()>;
    fn create_session(&mut self, room: &String) -> Result<usize>;
    fn remove_session(&mut self, session_id: usize) -> Result<()>;
    fn update_game(&mut self, game_id: usize, game_update: &Game) -> Result<()>;
    fn flip_card(&mut self, game_id: usize, coord: (usize, usize)) -> Result<Game>;
    fn get_game(&self, game_id: usize) -> Result<Game>;
    fn next_turn(&mut self, game_id: usize) -> Result<Game>;
}

#[derive(Clone)]
pub struct MemoryDatabaseTables {
    rooms: HashMap<String, Room>,
    games: HashMap<usize, Game>,
    sessions: HashMap<usize, ClientSession>,
}

impl MemoryDatabaseTables {
    pub fn new() -> MemoryDatabaseTables {
        MemoryDatabaseTables {
            rooms: HashMap::new(),
            games: HashMap::new(),
            sessions: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct MemoryDatabase {
    database: Arc<Mutex<MemoryDatabaseTables>>
}

impl MemoryDatabase {
    pub fn new() -> MemoryDatabase {
        MemoryDatabase {
            database: Arc::new(Mutex::new(MemoryDatabaseTables::new()))
        }
    }

    fn get_lock(&self) -> MutexGuard<MemoryDatabaseTables> {
        self.database.lock().unwrap()
    }

    fn get_lock_mut(&mut self) -> MutexGuard<MemoryDatabaseTables> {
        self.database.lock().unwrap()
    }
}

impl Database for MemoryDatabase {
    fn create_room(&mut self, name: &String) -> Result<String> {
        if self.get_lock().rooms.contains_key(name) {
            bail!("Room {} already exists!", name)
        }

        loop {
            let game_id = rand::thread_rng().gen();
            if self.get_lock().games.contains_key(&game_id) { continue; }

            self.get_lock().games.insert(game_id, Game::new());
            let new_room = Room::new(name.clone(), game_id);
            self.get_lock().rooms.insert(name.clone(), new_room.clone());
            return Ok(name.clone())
        }
    }

    fn remove_room(&mut self, name: &String) -> Result<()> {
        self.get_lock().rooms
            .remove(name)
            .context(format!("Failed to remove room with name '{}' because it did not exist.", name))
            .and(Ok(()))
    }

    fn get_room(&self, name: &String) -> Result<Room> {
        self.get_lock().rooms
            .get(name)
            .context(format!("Could not find room with name '{}'.", name))
            .cloned()
    }

    fn get_rooms(&self) -> Result<Vec<Room>> {
        Ok(self.get_lock().rooms.values().cloned().collect())
    }

    fn get_sessions(&self) -> Result<Vec<ClientSession>> {
        Ok(self.get_lock().sessions.values().cloned().collect())
    }

    fn get_session(&self, id: &usize) -> Result<ClientSession> {
        self.get_lock()
            .sessions
            .get(&id)
            .context(format!("Session with id {} does not exist.", id))
            .cloned()
    }

    fn create_session(&mut self, room: &String) -> Result<usize> {
        loop {
            let id = rand::thread_rng().gen();
            if self.get_lock().sessions.contains_key(&id) { continue; };

            let session = ClientSession::new(id, &room);
            self.get_lock().sessions.insert(id, session);

            return match self.get_lock().rooms.get_mut(room) {
                Some(val) => {
                    val.sessions.push(id);
                    Ok(id)
                },
                None => Err(anyhow!("Could not find room with name '{}'.", room))
            }
        } 
    }

    fn remove_session(&mut self, session_id: usize) -> Result<()> {
        let session = self.get_session(&session_id)?.clone();
        self.get_lock().sessions.remove(&session_id);

        self.get_lock_mut()
            .rooms
            .get_mut(&session.room)
            .context(format!("Could not find room with name '{}'.", &session.room))
            .and_then(|room| {
                if let Some(pos) = room.sessions.iter().position(|s| *s == session_id) {
                    room.sessions.swap_remove(pos);
                }
                Ok(())
            })
    }

    fn update_game(&mut self, game_id: usize, game_update: &Game) -> Result<()> {
        self.get_lock().games.get(&game_id)
            .context(format!("Cannot find game with id '{}'.", game_id))?;
        
        self.get_lock().games.insert(game_id, game_update.clone());
        Ok(())
    }

    fn get_game(&self, game_id: usize) -> Result<Game> {
        self.get_lock().games
            .get(&game_id)
            .context(format!("Could not find game with id '{}'.", game_id))
            .cloned()
    }

    fn update_session(&mut self, id: usize, session_update: &ClientSession) -> Result<()> {
        self.get_lock().sessions
            .get(&id)
            .context(format!("Could not find session with id '{}'.", id))?;
        
        self.get_lock().sessions.insert(id, session_update.clone());
        Ok(())
    }

    fn flip_card(&mut self, game_id: usize, coord: (usize, usize)) -> Result<Game> {
        let mut locked_database = self.get_lock();
        let game = locked_database.games
            .get_mut(&game_id)
            .context(format!("Could not find game with id '{}'.", game_id))?;
        let updated_game = game.flip_card(coord);
        *game = updated_game.clone();
        Ok(updated_game.clone())
    }

    fn next_turn(&mut self, game_id: usize) -> Result<Game> {
        let mut locked_database = self.get_lock();
        let game = locked_database.games
            .get_mut(&game_id)
            .context(format!("Could not find game with id '{}'.", game_id))?;
        let updated_game = game.next_turn();
        *game = updated_game.clone();
        Ok(updated_game)
    }
}

#[cfg(test)]
mod tests {
    use crate::client::ClientSession;

    use super::{Database, MemoryDatabase};

    #[test]
    fn creates_gets_removes_room() {
        let mut db = MemoryDatabase::new();
        let room_name = String::from("foo");
        db.create_room(&String::from("foo")).unwrap();
        let room = db.get_room(&room_name).unwrap();
        
        assert_eq!(room_name, room.name);
        assert_eq!(0, room.sessions.len());
        
        let game = db.get_game(room.game_id).unwrap();
        assert_eq!(5, game.board.len());

        db.remove_room(&room_name).unwrap();
        assert!(!db.get_rooms()
                    .unwrap()
                    .contains(&&room));

        let bad_get = db.get_room(&room_name);
        assert!(bad_get.is_err());
        let error_msg = bad_get.expect_err("Did not get an error").to_string();
        assert_eq!("Could not find room with name 'foo'.", error_msg);

        // updates game
        let new_game = game.flip_card((0, 0));
        db.update_game(room.game_id, &new_game).unwrap();
        let new_game = db.get_game(room.game_id).unwrap();
        assert!(new_game.board[0][0].flipped);
    }

    #[test]
    fn creates_deletes_session() {
        let mut db = MemoryDatabase::new();
        let room_name = String::from("foo");
        let username = String::from("foo_user");

        // Try to create a session in a room that doesn't exist
        let bad_result = db.create_session(&room_name);
        assert!(bad_result.is_err());
        let error_msg = bad_result.expect_err("Did not get an error").to_string();
        assert_eq!("Could not find room with name 'foo'.", error_msg);

        // Create room
        db.create_room(&room_name).unwrap();

        // Create session
        let new_session_id = db.create_session(&room_name).unwrap();
        let session_id_exists_in_sessions = db.get_sessions()
            .unwrap()
            .into_iter()
            .map(|s| s.id)
            .collect::<Vec<usize>>()
            .contains(&new_session_id);
        assert!(session_id_exists_in_sessions);

        // Add username
        let new_session = db.get_session(&new_session_id).unwrap();
        db.update_session(new_session_id, &ClientSession {username: username.clone(), ..new_session.clone()}).unwrap();

        let username_exists_in_sessions = db.get_sessions()
            .unwrap()
            .into_iter()
            .map(|s| s.username)
            .collect::<Vec<String>>()
            .contains(&username);
        assert!(username_exists_in_sessions);

        let username_exists_in_room = db.get_room(&room_name)
            .unwrap()
            .sessions
            .contains(&new_session_id);
        assert!(username_exists_in_room);

        db.remove_session(new_session_id).unwrap();
        let username_exists = db.get_sessions()
            .unwrap()
            .into_iter()
            .map(|s| s.username)
            .collect::<Vec<String>>()
            .contains(&username);
        assert!(!username_exists);

        let username_exists_in_room = db.get_room(&room_name)
            .unwrap()
            .sessions
            .contains(&new_session_id);
        assert!(!username_exists_in_room);
    }
}
