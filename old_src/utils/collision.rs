use glam::Vec2;

pub fn rect_contains_point(rect_size: Vec2, rect_position: Vec2, point: Vec2) -> bool {
    let rect_position = rect_position;
    let point = point;

    let x2 = rect_size.x + rect_position.x;
    let y2 = rect_size.y + rect_position.y;

    rect_position.x < point.x && point.x < x2 && rect_position.y < point.y && point.y < y2
}
