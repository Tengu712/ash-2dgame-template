//! 副作用に関するモジュール
//!
//! ここで言う副作用はゲーム状態更新時に発生する副作用のこと。
//! ゲーム更新関数の責務はゲーム状態の更新のみであり、それ以外はすべて副作用となる。
//!
//! ECSのようにゲーム状態から描画用データ等を収集しても良いが、
//! 正直ECSは大規模ゲームでない限り過剰だと思うので、
//! コマンドパターンを用いた単方向データフローっぽくしている。
//!
//! 本来はmainモジュールの役割。
//! でも肥大化しちゃうしファイル分割するとモジュールになっちゃうししゃあなし。

use crate::{
    System,
    graphics::descriptor::transform::{Camera, Instance},
    res::*,
};
use glam::*;

#[derive(Clone, Copy)]
pub enum Effect {
    Draw {
        position: Vec3,
        scaling: Vec2,
        color: Vec4,
        image: Resource,
        uv: Vec4,
    },
    LoadImage(Resource),
    ToggleFullscreen,
    UpdateCamera {
        position: Vec3,
        scaling: Vec3,
    },
}

#[derive(Default)]
pub struct EffectProcessor {
    /// インスタンスバッファにアップロードされるデータ列
    ///
    /// NOTE: なくても構わないがアロケーションコストを抑えるために。
    instances: Vec<Instance>,

    /// 更新すべきイメージディスクリプタの情報列
    ///
    /// NOTE: なくても構わないがアロケーションコストを抑えるために。
    images: Vec<(Resource, u32)>,
    /// イメージディスクリプタの更新状況
    ///
    /// インデックスはディスクリプタのオフセットを表す。
    updated_images: Vec<Resource>,
    /// そのフレームで出現したイメージの集合
    ///
    /// NOTE: どうせ数種類しかないのでHashSetよりVecの方が高速のはず。
    /// NOTE: なくても構わないがアロケーションコストを抑えるために。
    appeared_images: Vec<Resource>,
}

impl EffectProcessor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EffectProcessor {
    /// 副作用を処理するメソッド
    //
    // NOTE: EffectProcessorのメソッドとして処理関数を定義しているのは、
    //       副作用処理用の状態を持つ必要が(パフォーマンスのために)あり、
    //       かつ、処理する副作用と・副作用の適応対象をパラメータとするために、
    //       副作用処理用の状態が優位存在であると考えられるため。
    pub fn process(mut self, effects: &[Effect], mut system: System) -> (Self, System) {
        self.instances.clear();
        self.images.clear();
        self.appeared_images.clear();
        let mut camera = None;

        for effect in effects.iter().copied() {
            match effect {
                Effect::Draw {
                    position,
                    scaling,
                    color,
                    image,
                    uv,
                } => {
                    let tex_id =
                        match find_texture_id(image, &self.updated_images, &self.appeared_images) {
                            FindTexIdRes::Updated(i) => i,
                            FindTexIdRes::Overwrite(i) => {
                                self.images.push((image, i));
                                self.updated_images[i as usize] = image;
                                i
                            }
                            FindTexIdRes::Push(i) => {
                                self.images.push((image, i));
                                self.updated_images.push(image);
                                i
                            }
                        };
                    self.appeared_images.push(image);

                    self.instances.push(Instance {
                        transform: Mat4::from_translation(position)
                            * Mat4::from_scale(scaling.extend(1.0)),
                        color,
                        tex_id,
                        uv,
                    });
                }

                Effect::LoadImage(res) => system.gengine = system.gengine.load_image(&res),

                Effect::ToggleFullscreen => {
                    system.gengine = system.gengine.ensure_idle();
                    system.window.toggle_fullscreen();
                    system.gengine = system.gengine.recreate_swapchain(&system.window);
                }

                Effect::UpdateCamera { position, scaling } => {
                    camera = Some(Camera {
                        view: Mat4::from_translation(-position),
                        proj: Mat4::from_scale(scaling.recip()),
                    })
                }
            }
        }

        system.gengine =
            system
                .gengine
                .draw_frame(&system.window, &self.instances, &camera, &self.images);
        (self, system)
    }
}

enum FindTexIdRes {
    Updated(u32),
    Overwrite(u32),
    Push(u32),
}

fn find_texture_id(image: Resource, updateds: &[Resource], appeareds: &[Resource]) -> FindTexIdRes {
    // 更新済のイメージとして存在する
    if let Some(i) = updateds.iter().position(|v| v == &image) {
        FindTexIdRes::Updated(i as u32)
    }
    // そうでないなら未出現のスロットを上書きするように
    else if let Some(i) = updateds
        .iter()
        .position(|v| !appeareds.iter().any(|x| x == v))
    {
        FindTexIdRes::Overwrite(i as u32)
    }
    // それもできないなら追加するしかない
    else {
        FindTexIdRes::Push(updateds.len() as u32)
    }
}
