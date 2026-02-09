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
use std::vec::Drain;

mod chars;
mod texture;

use chars::CharsManageState;
use texture::TextureManageState;

#[allow(unused)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[allow(unused)]
pub enum Effect {
    Draw {
        position: Vec3,
        scaling: Vec2,
        color: Vec4,
        image: Resource,
        uv: Vec4,
    },
    DrawText {
        /// emスクエア高 [px]
        scale: u32,
        text: String,
        align: TextAlignment,
        position: Vec3,
        /// 1行の見かけの高さ
        line_height: f32,
        color: Vec4,
    },
    LoadImage(Resource),
    ToggleFullscreen,
    UpdateCamera {
        position: Vec3,
        scaling: Vec3,
    },
}

pub struct EffectProcessor {
    /// インスタンスバッファにアップロードされるデータ列
    ///
    /// NOTE: なくても構わないがアロケーションコストを抑えるために。
    instances: Vec<Instance>,
    /// インスタンスバッファにアップロードされるデータ列
    ///
    /// NOTE: 本当になくても構わないが型定義が煩雑になるので。
    camera: Option<Camera>,

    tex_state: TextureManageState,
    chars_state: CharsManageState,
}

impl EffectProcessor {
    pub fn new(system: &System) -> Self {
        Self {
            instances: Default::default(),
            camera: Default::default(),
            tex_state: Default::default(),
            chars_state: CharsManageState::new(system),
        }
    }
}

impl EffectProcessor {
    /// 副作用を処理するメソッド
    //
    // NOTE: EffectProcessorのメソッドとして処理関数を定義しているのは、
    //       副作用処理用の状態を持つ必要が(パフォーマンスのために)あり、
    //       かつ、処理する副作用と・副作用の適応対象をパラメータとするために、
    //       副作用処理用の状態が優位存在であると考えられるため。
    pub fn process(mut self, effects: Drain<'_, Effect>, mut system: System) -> (Self, System) {
        self.instances.clear();
        self.camera = None;
        self.tex_state = self.tex_state.clear();

        for effect in effects {
            (self, system) = self.process_effect(effect, system);
        }

        system.gengine = system.gengine.draw_frame(
            &system.window,
            &self.instances,
            &self.camera,
            &self.tex_state.images,
        );
        (self, system)
    }

    fn process_effect(mut self, effect: Effect, mut system: System) -> (Self, System) {
        match effect {
            Effect::Draw {
                position,
                scaling,
                color,
                image,
                uv,
            } => {
                let tex_id;
                (self.tex_state, tex_id) = self.tex_state.update(image);

                self.instances.push(Instance {
                    transform: Mat4::from_translation(position)
                        * Mat4::from_scale(scaling.extend(1.0)),
                    color,
                    tex_id,
                    uv,
                });
            }

            Effect::DrawText {
                scale,
                text,
                align,
                position,
                line_height,
                color,
            } => {
                let tex_id;
                (self.tex_state, tex_id) = self.tex_state.update(CHAR_ATLAS);

                // `position`をペンポイントとして普通にインスタンスデータ構築
                let mut pen_point = position.xy();
                let mut max_xy = pen_point;
                let start_i = self.instances.len();
                let start_x = pen_point.x;
                for c in text.chars() {
                    // TODO: 複数行に対応する。

                    let info;
                    (self.chars_state, info, system) = self.chars_state.update(c, scale, system);

                    let s = line_height / scale as f32;
                    let xy = Vec2::new(
                        pen_point.x + (info.x_offset + info.width / 2.0) * s,
                        pen_point.y + (info.y_offset + info.height / 2.0) * s,
                    );
                    let wh = Vec2::new(info.width * s, info.height * s);
                    self.instances.push(Instance {
                        transform: Mat4::from_translation(xy.extend(position.z))
                            * Mat4::from_scale(wh.extend(1.0)),
                        color,
                        tex_id,
                        uv: info.uv,
                    });

                    max_xy = max_xy.max(pen_point + wh);
                    pen_point.x += info.advance * s;
                }

                // アラインメント
                let cx = (start_x + max_xy.x) / 2.0;
                let dx = position.x - cx;
                match align {
                    TextAlignment::Left => (),
                    TextAlignment::Center => {
                        for instance in &mut self.instances[start_i..] {
                            instance.transform.w_axis.x += dx;
                        }
                    }
                    TextAlignment::Right => {
                        for instance in &mut self.instances[start_i..] {
                            instance.transform.w_axis.x += dx * 2.0;
                        }
                    }
                }
            }

            Effect::LoadImage(res) => system.gengine = system.gengine.load_image(&res),

            Effect::ToggleFullscreen => {
                system.gengine = system.gengine.ensure_idle();
                system.window.toggle_fullscreen();
                system.gengine = system.gengine.recreate_swapchain(&system.window);
            }

            Effect::UpdateCamera { position, scaling } => {
                self.camera = Some(Camera {
                    view: Mat4::from_translation(-position),
                    proj: Mat4::from_scale(scaling.recip()),
                })
            }
        }

        (self, system)
    }
}
