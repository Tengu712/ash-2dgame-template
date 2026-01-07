use super::*;
use std::f32::consts;

pub fn clamp_and_reflect(world: &mut World, _: &InputStates) {
    for (_, p, v) in world.components.ball_position_velocity() {
        if p.x <= 0.0 {
            p.x = 0.0;
            v.t = consts::PI - v.t;
        } else if p.x >= 640.0 {
            p.x = 640.0;
            v.t = consts::PI - v.t;
        }

        if p.y <= 0.0 {
            p.y = 0.0;
            v.t = -v.t;
        } else if p.y >= 480.0 {
            p.y = 480.0;
            v.t = -v.t;
        }
    }
}

pub fn reflect_with_player(world: &mut World, _: &InputStates) {
    let players = &world.components.players.0;
    let positions = &world.components.positions.0;
    let balls = &world.components.balls.0;
    let velocities = &mut world.components.velocities.0;

    const PLAYER_HALF_WIDTH: f32 = 40.0;
    const PLAYER_HALF_HEIGHT: f32 = 5.0;

    for pk in players.keys() {
        let Some(pp) = positions.get(pk) else {
            continue;
        };

        for bk in balls.keys() {
            let (Some(bp), Some(bv)) = (positions.get(bk), velocities.get_mut(bk)) else {
                continue;
            };

            let p_t = pp.y - PLAYER_HALF_HEIGHT;
            let p_b = pp.y + PLAYER_HALF_HEIGHT;
            let p_l = pp.x - PLAYER_HALF_WIDTH;
            let p_r = pp.x + PLAYER_HALF_WIDTH;

            if bp.y >= p_t && bp.y <= p_b && bp.x >= p_l && bp.x <= p_r {
                let rel = ((bp.x - pp.x) / PLAYER_HALF_WIDTH).clamp(-1.0, 1.0);
                bv.t = -(90.0 - rel * 60.0).to_radians();
            }
        }
    }
}
