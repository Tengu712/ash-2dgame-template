use super::*;
use glam::{Mat4, Vec3, Vec4};

/// `InstanceData`コンポーネントを更新するシステム
///
/// 次のコンポーネントに基づく:
/// - `Position`
/// - `ZIndex`
/// - `Scale`
/// - `Color`
///
/// NOTE: 最後のSystemとして追加すること。
pub fn update_instance(world: &mut World, _: &InputStates) {
    let components = &mut world.components;
    for (k, inst) in components.instances.0.iter_mut() {
        let x = components.positions.0.get(k).map_or(0.0, |p| p.x);
        let y = components.positions.0.get(k).map_or(0.0, |p| p.y);
        let z = components.zindices.0.get(k).map_or(-1.0, |z| -z.z);
        let s = components
            .scales
            .0
            .get(k)
            .map_or(Vec3::ONE, |s| Vec3::new(s.x, s.y, 1.0));
        let c = components
            .colors
            .0
            .get(k)
            .map_or(Vec4::ONE, |c| Vec4::new(c.r, c.g, c.b, c.a));
        inst.data.transform = Mat4::from_translation(Vec3::new(x, y, z)) * Mat4::from_scale(s);
        inst.data.color = c;
    }
}
