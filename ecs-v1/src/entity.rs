use serde::{Serialize, Deserialize};
use crate::Component;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entity {
    #[serde(skip)]
    pub id: usize,
    pub name: String,
    pub components: Vec<Box<dyn Component>>,
}

#[cfg(test)]
mod test {
    use crate::test_utils::*;
    use super::*;

    #[test]
    fn can_be_serialized_from_json() {
      can_go_from_json_to_mp::<Entity, _>("entity");
    }
}
