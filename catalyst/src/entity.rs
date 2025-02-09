use serde::{Deserialize, Serialize};
use std::fmt;

use crate::HashId;

/// Represents an entity from the server (scene, wearable, profile)
///
#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Entity {
    pub kind: EntityType,
    pub id: EntityId,
}

impl Entity {
    /// Constructs a new `Entity` with an `EntityType` and an id as a string.
    ///
    /// # Example
    ///
    /// ```
    /// use catalyst::Entity;
    /// use catalyst::EntityType;
    /// let entity = Entity::new(EntityType::Scene, "bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    ///
    /// assert_eq!(entity.kind, EntityType::Scene);
    /// assert_eq!(entity.id.hash(), "bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    /// ```
    pub fn new<T>(kind: EntityType, id: T) -> Entity
    where
        T: AsRef<str>,
    {
        Entity {
            kind,
            id: EntityId::new(id),
        }
    }

    /// Constructs a new `Profile` entity with id as a string.
    ///
    /// # Example
    ///
    /// ```
    /// use catalyst::Entity;
    /// use catalyst::EntityType;
    /// let entity = Entity::profile("bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    ///
    /// assert_eq!(entity.kind, EntityType::Profile);
    /// assert_eq!(entity.id.hash(), "bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    /// ```
    pub fn profile<T>(id: T) -> Entity
    where
        T: AsRef<str>,
    {
        Entity::new(EntityType::Profile, id)
    }

    /// Constructs a new `Scene` entity with id as a string.
    ///
    /// # Example
    ///
    /// ```
    /// use catalyst::Entity;
    /// use catalyst::EntityType;
    /// let entity = Entity::scene("bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    ///
    /// assert_eq!(entity.kind, EntityType::Scene);
    /// assert_eq!(entity.id.hash(), "bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    /// ```
    pub fn scene<T>(id: T) -> Entity
    where
        T: AsRef<str>,
    {
        Entity::new(EntityType::Scene, id)
    }

    /// Constructs a new `Wearable` entity with id as a string.
    ///
    /// # Example
    ///
    /// ```
    /// use catalyst::Entity;
    /// use catalyst::EntityType;
    /// let entity = Entity::wearable("bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    ///
    /// assert_eq!(entity.kind, EntityType::Wearable);
    /// assert_eq!(entity.id.hash(), "bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    /// ```
    pub fn wearable<T>(id: T) -> Entity
    where
        T: AsRef<str>,
    {
        Entity::new(EntityType::Wearable, id)
    }
}

/// All available entity types
///
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub enum EntityType {
    #[serde(rename = "profile")]
    Profile,
    #[serde(rename = "scene")]
    Scene,
    #[serde(rename = "wearable")]
    Wearable,
    #[serde(rename = "emote")]
    Emote,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let serialization = match self {
            EntityType::Profile => "profile",
            EntityType::Scene => "scene",
            EntityType::Wearable => "wearable",
            EntityType::Emote => "emote",
        };
        write!(f, "{}", serialization)
    }
}

/// Represents a hash that is used in the context of an entity id.
///
/// This struct implements `Display` to simplify the formatting of urls and messages.
///
/// ```
/// let entityId = catalyst::EntityId::new("a-missing-entity");
/// let message = format!("entity missing: {}", entityId);
/// assert_eq!(message, "entity missing: a-missing-entity");
/// ```
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct EntityId(pub HashId);

impl EntityId {
    /// Constructs a new entity id with id as a string.
    ///
    /// # Example
    ///
    /// ```
    /// use catalyst::EntityId;
    /// let id = EntityId::new("bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    ///
    /// assert_eq!(id.hash(), "bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    /// ```
    pub fn new<T>(id: T) -> EntityId
    where
        T: AsRef<str>,
    {
        EntityId(id.as_ref().to_string())
    }

    /// Returns the hash for this id
    ///
    /// # Example
    ///
    /// ```
    /// use catalyst::EntityId;
    /// let id = EntityId::new("bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    ///
    /// assert_eq!(id.hash(), "bafkreiabfxgn375iwwgtx2i5zhhtge2affusbt7sndnf7wqbkeuz4f36ki");
    /// ```
    pub fn hash(&self) -> &HashId {
        &self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn entity_can_create_a_scene() {
        let scene = Entity::scene("id");
        assert_eq!(scene.kind, EntityType::Scene);
        assert_eq!(scene.id, EntityId::new("id"));
    }

    #[test]
    fn entity_can_create_a_profile() {
        let scene = Entity::profile("id");
        assert_eq!(scene.kind, EntityType::Profile);
        assert_eq!(scene.id, EntityId::new("id"));
    }

    #[test]
    fn entity_can_create_a_wearable() {
        let scene = Entity::wearable("id");
        assert_eq!(scene.kind, EntityType::Wearable);
        assert_eq!(scene.id, EntityId::new("id"));
    }

    #[test]
    fn entity_id_implements_display() {
        let id = EntityId::new("id");
        let id_string = format!("{}", id);
        assert_eq!(id_string, "id");
    }

    #[test]
    fn entity_id_implements_hash() {
        let id = EntityId::new("a-hash");
        assert_eq!(id.hash(), "a-hash");
    }

    #[test]
    fn entity_type_deserializes_correctly() {
        assert_eq!(
            EntityType::Profile,
            serde_json::from_str("\"profile\"").unwrap()
        );
        assert_eq!(
            EntityType::Scene,
            serde_json::from_str("\"scene\"").unwrap()
        );
        assert_eq!(
            EntityType::Wearable,
            serde_json::from_str("\"wearable\"").unwrap()
        );
    }
}
