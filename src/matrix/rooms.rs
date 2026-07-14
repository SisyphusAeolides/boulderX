use std::collections::HashMap;
use matrix_sdk::ruma::OwnedRoomId;

#[derive(Debug, Clone)]
pub struct MatrixRoom {
    pub room_id: OwnedRoomId,
    pub display_name: String,
    pub unread_count: u32,
    pub topic: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct RoomRegistry {
    rooms: HashMap<String, MatrixRoom>,
}

impl RoomRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, room_id: String, name: String) {
        if let Ok(id) = room_id.parse::<OwnedRoomId>() {
            self.rooms.insert(room_id, MatrixRoom {
                room_id: id,
                display_name: name,
                unread_count: 0,
                topic: None,
            });
        }
    }

    pub fn get(&self, room_id: &str) -> Option<&MatrixRoom> {
        self.rooms.get(room_id)
    }

    pub fn get_mut(&mut self, room_id: &str) -> Option<&mut MatrixRoom> {
        self.rooms.get_mut(room_id)
    }

    pub fn all(&self) -> Vec<&MatrixRoom> {
        let mut rooms: Vec<&MatrixRoom> = self.rooms.values().collect();
        rooms.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
        rooms
    }

    pub fn remove(&mut self, room_id: &str) {
        self.rooms.remove(room_id);
    }

    pub fn increment_unread(&mut self, room_id: &str) {
        if let Some(r) = self.rooms.get_mut(room_id) {
            r.unread_count += 1;
        }
    }

    pub fn clear_unread(&mut self, room_id: &str) {
        if let Some(r) = self.rooms.get_mut(room_id) {
            r.unread_count = 0;
        }
    }
}
