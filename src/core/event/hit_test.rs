use crate::core::{
    arena::{NodeArena, node::NodeId},
    layout::{Point, Rect},
};

pub fn hit_test(arena: &NodeArena, root: NodeId, point: Point<f32>) -> Vec<NodeId> {
    let mut path = Vec::new();
    hit_test_inner(arena, root, point, Point::xy(0.0, 0.0), &mut path);
    path
}

fn hit_test_inner(
    arena: &NodeArena,
    node_id: NodeId,
    point: Point<f32>,
    parent_offset: Point<f32>,
    path: &mut Vec<NodeId>,
) -> bool {
    let layout = match arena.get_layout(node_id) {
        Ok(l) => l,
        Err(_) => return false,
    };

    let layout = layout.unrounded;

    let bounds = Rect::xywh(
        parent_offset.x + layout.location.x,
        parent_offset.y + layout.location.y,
        layout.size.width,
        layout.size.height,
    );

    if !bounds.includes(point) {
        return false;
    }

    let children = match arena.get_children(node_id) {
        Ok(c) => c.clone(),
        Err(_) => Vec::new(),
    };

    let child_offset = Point::xy(bounds.location.x, bounds.location.y);

    for &child_id in children.iter().rev() {
        if hit_test_inner(arena, child_id, point, child_offset, path) {
            path.insert(0, node_id);
            return true;
        }
    }

    path.push(node_id);
    true
}
