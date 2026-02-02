use super::*;
use crate::input::Key;
use glam::Vec2;
use std::f32::consts::{FRAC_PI_4, PI};

const BAR_Y: f32 = 0.75;
const BAR_SIZE: Vec2 = Vec2::new(0.25, 0.05);
const BAR_SPEED: f32 = 0.025;

const BALL_SIZE: Vec2 = Vec2::new(0.03, 0.03);
const BALL_SPEED: f32 = 0.015;

const BLOCK_ROWS: usize = 3;
const BLOCK_COLS: usize = 5;
const BLOCK_SIZE: Vec2 = Vec2::new(0.18, 0.08);
const BLOCK_START_Y: f32 = -0.5;
const BLOCK_SPACING: Vec2 = Vec2::new(0.22, 0.12);

#[derive(Clone, Copy)]
struct Bar {
    x: f32,
}

impl Bar {
    fn new() -> Self {
        Self { x: 0.0 }
    }

    fn update(self, istates: &InputStates) -> Self {
        let r = (istates.get(Key::Right) > 0) as i32;
        let l = (istates.get(Key::Left) > 0) as i32;
        let dx = (r - l) as f32 * BAR_SPEED;
        let x = (self.x + dx).clamp(-1.0 + BAR_SIZE.x / 2.0, 1.0 - BAR_SIZE.x / 2.0);
        Self { x }
    }

    fn pos(&self) -> Vec2 {
        Vec2::new(self.x, BAR_Y)
    }
}

#[derive(Clone, Copy)]
struct Ball {
    pos: Vec2,
    angle: f32,
}

impl Ball {
    fn new() -> Self {
        Self {
            pos: Vec2::new(0.0, 0.3),
            angle: FRAC_PI_4,
        }
    }

    fn update(self, bar: &Bar) -> Self {
        let mut angle = self.angle;
        if self.pos.x <= -1.0 || self.pos.x >= 1.0 {
            angle = PI - angle;
        }
        if self.pos.y <= -1.0 {
            angle = -angle;
        }
        if collides_point_rect(self.pos, bar.pos(), BAR_SIZE) {
            angle = reflect_radial(self.pos, bar.pos());
        }
        Self {
            pos: self.pos + Vec2::new(angle.cos(), angle.sin()) * BALL_SPEED,
            angle,
        }
    }
}

#[derive(Clone, Copy)]
struct Block {
    pos: Vec2,
    alive: bool,
}

fn create_blocks() -> Vec<Block> {
    (0..BLOCK_ROWS)
        .flat_map(|row| {
            (0..BLOCK_COLS).map(move |col| {
                let x = (col as f32 - (BLOCK_COLS - 1) as f32 / 2.0) * BLOCK_SPACING.x;
                let y = BLOCK_START_Y + row as f32 * BLOCK_SPACING.y;
                Block {
                    pos: Vec2::new(x, y),
                    alive: true,
                }
            })
        })
        .collect()
}

fn collide_ball_blocks(ball: Ball, mut blocks: Vec<Block>) -> (Ball, Vec<Block>) {
    let hit_pos = blocks.iter_mut().find_map(|block| {
        if block.alive && collides_point_rect(ball.pos, block.pos, BLOCK_SIZE) {
            block.alive = false;
            Some(block.pos)
        } else {
            None
        }
    });
    let ball = hit_pos.map_or(ball, |hit_pos| Ball {
        angle: reflect_radial(ball.pos, hit_pos),
        ..ball
    });
    (ball, blocks)
}

/// 放射状に反射する角度を求める関数
fn reflect_radial(org: Vec2, trg: Vec2) -> f32 {
    let dir = (org - trg).normalize();
    dir.y.atan2(dir.x)
}

/// 点と長方形の衝突を判定する関数
fn collides_point_rect(p: Vec2, r_pos: Vec2, r_size: Vec2) -> bool {
    (p.x - r_pos.x).abs() < r_size.x && (p.y - r_pos.y).abs() < r_size.y
}

pub struct States {
    bar: Bar,
    ball: Ball,
    blocks: Vec<Block>,
}

impl States {
    pub fn new() -> Self {
        Self {
            bar: Bar::new(),
            ball: Ball::new(),
            blocks: create_blocks(),
        }
    }

    pub fn update(self, istates: &InputStates) -> (GameState, RenderingInfo) {
        let bar = self.bar.update(istates);
        let ball = self.ball.update(&bar);
        let (ball, blocks) = collide_ball_blocks(ball, self.blocks);

        // 負け
        if ball.pos.y > 1.0 {
            return title::States::new().update(istates);
        }
        // 勝ち
        if blocks.iter().all(|b| !b.alive) {
            return title::States::new().update(istates);
        }

        let mut instances = Vec::with_capacity(1 + 1 + BLOCK_ROWS * BLOCK_COLS);
        // バー
        instances.push(Instance {
            position: Vec3::new(bar.x, BAR_Y, 0.0),
            scaling: Vec2::new(BAR_SIZE.x, BAR_SIZE.y),
            ..Default::default()
        });
        // ボール
        instances.push(Instance {
            position: Vec3::new(ball.pos.x, ball.pos.y, 0.0),
            scaling: Vec2::new(BALL_SIZE.x, BALL_SIZE.y),
            ..Default::default()
        });
        // ブロック
        blocks.iter().filter(|block| block.alive).for_each(|block| {
            instances.push(Instance {
                position: Vec3::new(block.pos.x, block.pos.y, 0.0),
                scaling: Vec2::new(BLOCK_SIZE.x, BLOCK_SIZE.y),
                ..Default::default()
            })
        });

        (
            GameState::Game(Box::new(Self { bar, ball, blocks })),
            RenderingInfo {
                instances,
                ..Default::default()
            },
        )
    }
}
