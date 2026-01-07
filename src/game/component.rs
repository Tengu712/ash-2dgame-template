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

macro_rules! define_query_method {
    ($name1:ident $cname1:ident: $sname1:ty, $name2:ident $cname2:ident: $sname2:ty) => {
        impl ComponentStorages {
            cat_ids! {
                /// クエリメソッド (2要素)
                ///
                /// コンポーネントA, Bについて、(&mut A, &mut B)のイテレータを返す。
                /// A優先でイテレートされる。
                pub fn [|$name1 _ $name2|](&mut self) -> impl Iterator<Item = (&mut $sname1, &mut $sname2)> + '_ {
                    let $cname2 = &mut self.$cname2.0 as *mut HashMap<Entity, $sname2>;
                    self.$cname1.0.iter_mut().filter_map(move |(entity, $name1)| {
                        // SAFETY: 各Entityは一意であり、
                        //         異なるEntityのPositionとVelocityへの可変参照は重複しない。
                        unsafe {
                            (*$cname2).get_mut(entity).map(|$name2| ($name1, $name2))
                        }
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

define_query_method!(position positions: Position, velocity velocities: Velocity);
define_query_method!(player players: Player, position positions: Position);
define_query_method!(player players: Player, velocity velocities: Velocity);
