use crate::collision_type::CollisionType;
use crate::Size;
use crate::Vec2;
use core::any::Any;
use serde::{Deserialize, Serialize};

use crate::Component;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BoxCollider {
    #[serde(default)]
    pub collision_type: CollisionType,
    #[serde(default)]
    pub center: Vec2<i32>,
    #[serde(default)]
    pub size: Size,
}

#[typetag::serde]
impl Component for BoxCollider {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn can_be_serialized_from_json() {
        can_go_from_json_to_mp::<BoxCollider, _>("components/box_collider");
    }

    #[test]
    fn supports_optional_values_with_defaults() {
        let json = load_json_fixture("components/box_collider_optional").unwrap();
        let result: BoxCollider = serde_json::from_str(&json).unwrap();
        assert_eq!(
            result,
            BoxCollider {
                collision_type: CollisionType::Solid,
                center: Vec2 { x: 0, y: 0 },
                size: Size {
                    width: 1,
                    height: 1
                },
            }
        )
    }
}
