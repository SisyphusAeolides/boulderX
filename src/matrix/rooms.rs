use matrix_sdk::ruma::OwnedRoomId;
use std::collections::HashMap;

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
            // Preserve unread if re-inserting after sync seed.
            let unread = self
                .rooms
                .get(&room_id)
                .map(|r| r.unread_count)
                .unwrap_or(0);
            self.rooms.insert(
                room_id,
                MatrixRoom {
                    room_id: id,
                    display_name: name,
                    unread_count: unread,
                    topic: None,
                },
            );
        }
    }

    pub fn get(&self, room_id: &str) -> Option<&MatrixRoom> {
        self.rooms.get(room_id)
    }

    pub fn get_mut(&mut self, room_id: &str) -> Option<&mut MatrixRoom> {
        self.rooms.get_mut(room_id)
    }

    pub fn find_by_display_name(&self, name: &str) -> Option<&MatrixRoom> {
        self.rooms.values().find(|r| r.display_name == name)
    }

    pub fn room_id_for_display_name(&self, name: &str) -> Option<String> {
        self.find_by_display_name(name)
            .map(|r| r.room_id.to_string())
    }

    pub fn all(&self) -> Vec<&MatrixRoom> {
        let mut rooms: Vec<&MatrixRoom> = self.rooms.values().collect();
        rooms.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        });
        rooms
    }

    pub fn remove(&mut self, room_id: &str) {
        self.rooms.remove(room_id);
    }

    pub fn increment_unread(&mut self, room_id: &str) {
        if let Some(r) = self.rooms.get_mut(room_id) {
            r.unread_count = r.unread_count.saturating_add(1);
        }
    }

    pub fn clear_unread(&mut self, room_id: &str) {
        if let Some(r) = self.rooms.get_mut(room_id) {
            r.unread_count = 0;
        }
    }

    pub fn clear_unread_by_display_name(&mut self, name: &str) {
        if let Some(id) = self.room_id_for_display_name(name) {
            self.clear_unread(&id);
        }
    }

    pub fn len(&self) -> usize {
        self.rooms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rooms.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_ignores_invalid_room_id() {
        let mut reg = RoomRegistry::new();
        reg.insert("not-a-room".into(), "Bad".into());
        assert!(reg.is_empty());
    }

    #[test]
    fn insert_and_unread_cycle() {
        let mut reg = RoomRegistry::new();
        // Valid room id form: !localpart:server
        let id = "!abc:example.org".to_string();
        reg.insert(id.clone(), "General".into());
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.get(&id).unwrap().unread_count, 0);
        reg.increment_unread(&id);
        reg.increment_unread(&id);
        assert_eq!(reg.get(&id).unwrap().unread_count, 2);
        reg.clear_unread_by_display_name("General");
        assert_eq!(reg.get(&id).unwrap().unread_count, 0);
    }

    #[test]
    fn all_sorts_by_display_name() {
        let mut reg = RoomRegistry::new();
        reg.insert("!z:example.org".into(), "zeta".into());
        reg.insert("!a:example.org".into(), "Alpha".into());
        let names: Vec<_> = reg.all().iter().map(|r| r.display_name.as_str()).collect();
        assert_eq!(names, vec!["Alpha", "zeta"]);
    }
}
