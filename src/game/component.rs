use super::Entity;
use crate::graphics::descriptor::transform::Instance;
use procmacro::cat_ids;
use std::collections::HashMap;

pub trait Component: Default + Sized + 'static {}

#[derive(Default)]
pub struct ComponentStorage<T: Component>(pub HashMap<Entity, T>);

macro_rules! define_components {
    {$($(#[$meta:meta])* $sname:ident { $($tt:tt)* } $name:ident $cname:ident;)*} => {
        $(
            #[derive(Default)]
            pub struct $sname {
                $($tt)*
            }
            impl Component for $sname {}
        )*

        #[derive(Default)]
        pub struct ComponentStorages {
            $(pub $cname: ComponentStorage<$sname>,)*
        }

        impl ComponentStorages {
            cat_ids! {
                $(
                    #[allow(dead_code)]
                    pub fn [|insert_ $name|](&mut self, entity: Entity, value: $sname) {
                        self.$cname.0.insert(entity, value);
                    }
                )*
            }

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
    Player {} player players;

    Position { pub x: f32, pub y: f32 } position positions;

    /// Z値
    ///
    /// 上下を順序付けるためのもの。
    /// 0に近いほど上に描画される。
    ///
    /// NOTE: 範囲[0, 1]内の値を指定すること。
    ZIndex { pub z: f32 } zindex zindices;

    Scale { pub x: f32, pub y: f32 } scale scales;

    Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 } color colors;

    /// レンダリングエンジン向けのデータ
    ///
    /// NOTE: 描画されるエンティティは必ずこのコンポーネントを持つこと。
    ///       また、Systemで正しく更新すること。
    InstanceData { pub data: Instance } instance instances;

    Velocity { pub r: f32, pub t: f32 } velocity velocities;
}
