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

macro_rules! define_query_methods {
    ($mname:ident $mcname:ident: &mut $mty:ty, $rname:ident $rcname:ident: $rty:ty) => {
        impl ComponentStorages {
            cat_ids! {
                pub fn [|$mname _ $rname|](&mut self) -> impl Iterator<Item = (&mut $mty, &$rty)> + '_ {
                    let $rcname = &self.$rcname.0;
                    self.$mcname
                        .0
                        .iter_mut()
                        .filter_map(move |(entity, $mname)| {
                            $rcname.get(entity).map(|$rname| ($mname, $rname))
                        })
                }
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

define_query_methods!(position positions: &mut Position, velocity velocities: Velocity);
