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
pub enum TextHorizontalAlignment {
    Left,
    Center,
    Right,
}

impl TextHorizontalAlignment {
    fn align(&self, width: f32, instances: &mut [Instance]) {
        match self {
            Self::Left => (),
            Self::Center => {
                for instance in instances {
                    instance.transform.w_axis.x -= width / 2.0;
                }
            }
            Self::Right => {
                for instance in instances {
                    instance.transform.w_axis.x -= width;
                }
            }
        }
    }
}

#[allow(unused)]
pub enum TextVerticalAlignment {
    Top,
    Center,
    Bottom,
}

impl TextVerticalAlignment {
    fn align(&self, ascent: f32, height: f32, instances: &mut [Instance]) {
        match self {
            Self::Top => {
                for instance in instances {
                    instance.transform.w_axis.y += ascent;
                }
            }
            Self::Center => {
                for instance in instances {
                    instance.transform.w_axis.y += ascent;
                    instance.transform.w_axis.y -= height / 2.0;
                }
            }
            Self::Bottom => {
                for instance in instances {
                    instance.transform.w_axis.y += ascent;
                    instance.transform.w_axis.y -= height;
                }
            }
        }
    }
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
        halign: TextHorizontalAlignment,
        valign: TextVerticalAlignment,
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
                halign,
                valign,
                position,
                line_height,
                color,
            } => {
                let tex_id;
                (self.tex_state, tex_id) = self.tex_state.update(CHAR_ATLAS);

                let s = line_height / scale as f32;
                let ascent = self.chars_state.ascent(scale) * s;
                let line_gap = self.chars_state.line_gap(scale) * s;
                let start_i = self.instances.len();
                let mut pp_y = position.y;
                let mut line_count = 1;
                let mut should_end = false;
                let mut chars = text.chars();

                // すべての行について
                while !should_end {
                    should_end = true;
                    let start_i = self.instances.len();
                    let mut pp_x = position.x;
                    let mut prev = None;

                    // 各行のインスタンスデータ構築
                    for c in &mut chars {
                        should_end = false;
                        if c == '\n' {
                            line_count += 1;
                            break;
                        }

                        // カーニング考慮
                        if let Some(prev) = prev {
                            pp_x += self.chars_state.kern(scale, prev, c) * s;
                        }
                        prev = Some(c);

                        let info;
                        (self.chars_state, info, system) =
                            self.chars_state.update(c, scale, system);

                        let w = info.width * s;
                        let h = info.height * s;
                        let x = pp_x + w / 2.0 + info.x_offset * s;
                        let y = pp_y + h / 2.0 + info.y_offset * s;
                        self.instances.push(Instance {
                            transform: Mat4::from_translation(Vec3::new(x, y, position.z))
                                * Mat4::from_scale(Vec3::new(w, h, 1.0)),
                            color,
                            tex_id,
                            uv: info.uv,
                        });

                        pp_x += info.advance * s;
                    }

                    // 各行を水平方向にアラインメント
                    halign.align(pp_x - position.x, &mut self.instances[start_i..]);

                    // 改行
                    pp_y += line_height + line_gap;
                }

                // すべての文字を垂直方向にアラインメント
                let height = (line_height + line_gap) * line_count as f32 - line_gap;
                valign.align(ascent, height, &mut self.instances[start_i..]);
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
