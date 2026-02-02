use super::*;
use crate::input::Key;
use glam::Vec2;
use std::f32::consts::{FRAC_PI_4, PI};

const BAR_Y: f32 = 420.0;
const BAR_SIZE: Vec2 = Vec2::new(80.0, 12.0);
const BAR_SPEED: f32 = 8.0;

const BALL_SIZE: Vec2 = Vec2::new(10.0, 10.0);
const BALL_SPEED: f32 = 5.0;

const BLOCK_ROWS: usize = 3;
const BLOCK_COLS: usize = 5;
const BLOCK_SIZE: Vec2 = Vec2::new(58.0, 20.0);
const BLOCK_START_Y: f32 = 120.0;
const BLOCK_SPACING: Vec2 = Vec2::new(70.0, 30.0);

#[derive(Clone, Copy)]
struct Bar {
    x: f32,
}

impl Bar {
    fn new() -> Self {
        Self {
            x: VIRTUAL_WIDTH_HALF,
        }
    }

    fn update(self, istates: &InputStates) -> Self {
        let r = (istates.get(Key::Right) > 0) as i32;
        let l = (istates.get(Key::Left) > 0) as i32;
        let dx = (r - l) as f32 * BAR_SPEED;
        let x = (self.x + dx).clamp(BAR_SIZE.x / 2.0, VIRTUAL_WIDTH - BAR_SIZE.x / 2.0);
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
            pos: Vec2::new(VIRTUAL_WIDTH_HALF, 312.0),
            angle: FRAC_PI_4,
        }
    }

    fn update(self, bar: &Bar) -> Self {
        let mut angle = self.angle;
        if self.pos.x <= 0.0 || self.pos.x >= VIRTUAL_WIDTH {
            angle = PI - angle;
        }
        if self.pos.y <= 0.0 {
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
                let x = VIRTUAL_WIDTH_HALF
                    + (col as f32 - (BLOCK_COLS - 1) as f32 / 2.0) * BLOCK_SPACING.x;
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
    (p.x - r_pos.x).abs() < r_size.x / 2.0 && (p.y - r_pos.y).abs() < r_size.y / 2.0
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
        if ball.pos.y > VIRTUAL_HEIGHT {
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
