use super::Entity;
use std::collections::HashMap;

pub trait Component: Default + Sized + 'static {}

#[derive(Default)]
pub struct ComponentStorage<T: Component>(pub HashMap<Entity, T>);

macro_rules! define_components {
    {$($sname:ident { $($field:ident: $ty:ty),* $(,)? } collected $cname:ident;)*} => {
        $(
            #[derive(Default)]
            pub struct $sname {
                $(pub $field: $ty,)*
            }
            impl Component for $sname {}
        )*

        #[derive(Default)]
        pub struct ComponentStorages {
            $(pub $cname: ComponentStorage<$sname>,)*
        }

        impl ComponentStorages {
            pub fn destroy_entity(&mut self, entity: Entity) {
                $(self.$cname.0.remove(&entity);)*
            }

            pub fn clear(&mut self) {
                $(self.$cname.0.clear();)*
            }
        }
    };
}

define_components! {
    Position { x: f32, y: f32 } collected positions;

    Velocity { r: f32, t: f32 } collected velocities;

    Scale { x: f32, y: f32 } collected scales;

    Player {} collected players;
}
