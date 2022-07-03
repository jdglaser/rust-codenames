use std::{collections::{hash_map, HashMap}, sync::{Mutex, Arc, MutexGuard}};

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
    fn get_game(&self, game_id: usize) -> Result<Game>;
}

#[derive(Clone)]
pub struct MemoryDatabaseRepo {
    database: Arc<Mutex<MemoryDatabase>>
}

impl MemoryDatabaseRepo {
    pub fn new() -> MemoryDatabaseRepo {
        MemoryDatabaseRepo { database: Arc::new(Mutex::new(MemoryDatabase::new())) }
    }

    fn get_lock(&self) -> MutexGuard<MemoryDatabase> {
        self.database.lock().unwrap()
    }
}

impl Database for MemoryDatabaseRepo {
    fn create_room(&mut self, name: &String) -> Result<String> {
        self.get_lock().create_room(name)
    }

    fn remove_room(&mut self, name: &String) -> Result<()> {
        self.get_lock().remove_room(name)
    }

    fn get_room(&self, name: &String) -> Result<Room> {
        self.get_lock().get_room(name)
    }

    fn get_rooms(&self) -> Result<Vec<Room>> {
        self.get_lock().get_rooms()
    }

    fn get_sessions(&self) -> Result<Vec<ClientSession>> {
        self.get_lock().get_sessions()
    }

    fn get_session(&self, id: &usize) -> Result<ClientSession> {
        self.get_lock().get_session(id)
    }

    fn update_session(&mut self, id: usize, session_update: &ClientSession) -> Result<()> {
        self.get_lock().update_session(id, session_update)
    }

    fn create_session(&mut self, room: &String) -> Result<usize> {
        self.get_lock().create_session(room)
    }

    fn remove_session(&mut self, session_id: usize) -> Result<()> {
        self.get_lock().remove_session(session_id)
    }

    fn update_game(&mut self, game_id: usize, game_update: &Game) -> Result<()> {
        self.get_lock().update_game(game_id, game_update)
    }

    fn get_game(&self, game_id: usize) -> Result<Game> {
        self.get_lock().get_game(game_id)
    }
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

    fn get_room_mut(&mut self, name: &String) -> Result<&mut Room> {
        self.rooms
            .get_mut(name)
            .context(format!("Could not find room with name '{}'.", name))
    }
}

impl Database for MemoryDatabase {
    fn create_room(&mut self, name: &String) -> Result<String> {
        if self.rooms.contains_key(name) {
            bail!("Room {} already exists!", name)
        }

        loop {
            let game_id = rand::thread_rng().gen();
            if self.games.contains_key(&game_id) { continue; }

            self.games.insert(game_id, Game::new());
            let new_room = Room::new(name.clone(), game_id);
            self.rooms.insert(name.clone(), new_room.clone());
            return Ok(name.clone())
        }
    }

    fn remove_room(&mut self, name: &String) -> Result<()> {
        self.rooms
            .remove(name)
            .context(format!("Failed to remove room with name '{}' because it did not exist.", name))
            .and(Ok(()))
    }

    fn get_room(&self, name: &String) -> Result<Room> {
        self.rooms
            .get(name)
            .context(format!("Could not find room with name '{}'.", name))
            .cloned()
    }

    fn get_rooms(&self) -> Result<Vec<Room>> {
        Ok(self.rooms.values().cloned().collect())
    }

    fn get_sessions(&self) -> Result<Vec<ClientSession>> {
        Ok(self.sessions.values().cloned().collect())
    }

    fn get_session(&self, id: &usize) -> Result<ClientSession> {
        self.sessions
            .get(&id)
            .context(format!("Session with id {} does not exist.", id))
            .cloned()
    }

    fn create_session(&mut self, room: &String) -> Result<usize> {
        loop {
            let id = rand::thread_rng().gen();
            if self.sessions.contains_key(&id) { continue; };

            let session = ClientSession::new(id, &room);
            self.sessions.insert(id, session);

            return match self.rooms.get_mut(room) {
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
        self.sessions.remove(&session_id);
        let room = self.get_room_mut(&session.room)?;
        
        if let Some(pos) = room.sessions.iter().position(|s| *s == session_id) {
            room.sessions.swap_remove(pos);
        }

        Ok(())
    }

    fn update_game(&mut self, game_id: usize, game_update: &Game) -> Result<()> {
        self.games.get(&game_id)
            .context(format!("Cannot find game with id '{}'.", game_id))?;
        
        self.games.insert(game_id, game_update.clone());
        Ok(())
    }

    fn get_game(&self, game_id: usize) -> Result<Game> {
        self.games
            .get(&game_id)
            .context(format!("Could not find game with id '{}'.", game_id))
            .cloned()
    }

    fn update_session(&mut self, id: usize, session_update: &ClientSession) -> Result<()> {
        self.sessions
            .get(&id)
            .context(format!("Could not find session with id '{}'.", id))?;
        
        self.sessions.insert(id, session_update.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::client::ClientSession;

    use super::{Database, MemoryDatabase};
    use super::{Game};

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

        db.remove_room(&room_name).unwrap();
        assert!(!db.get_rooms()
                    .unwrap()
                    .contains(&&room));

        let bad_get = db.get_room(&room_name);
        assert!(bad_get.is_err());
        let error_msg = bad_get.expect_err("Did not get an error").to_string();
        assert_eq!("Could not find room with name 'foo'.", error_msg);

        // updates game
        let new_board = game.flip_card((0, 0));
        db.update_game(room.game_id, &Game {board: new_board, ..game}).unwrap();
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
