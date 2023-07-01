use glam::Vec2;
use render_buddy::rect::Rect;

use crate::components::line::Line2D;

pub fn line_line_collision(a: Line2D, b: Line2D) -> Option<Vec2> {
    // calculate the distance to intersection point
    #[rustfmt::skip]
    let u_a = 
          ((b.1.x - b.0.x) * (a.0.y - b.0.y) - (b.1.y - b.0.y) * (a.0.x - b.0.x))
        / ((b.1.y - b.0.y) * (a.1.x - a.0.x) - (b.1.x - b.0.x) * (a.1.y - a.0.y));
    #[rustfmt::skip]
    let u_b = 
          ((a.1.x - a.0.x) * (a.0.y - b.0.y) - (a.1.y - a.0.y) * (a.0.x - b.0.x))
        / ((b.1.y - b.0.y) * (a.1.x - a.0.x) - (b.1.x - b.0.x) * (a.1.y - a.0.y));

    if u_a >= 0. && u_a <= 1f32 && u_b >= 0. && u_b <= 1f32 {
        let intersection_x = a.0.x + (u_a * (a.1.x - a.0.x));
        let intersection_y = a.0.y + (u_a * (a.1.y - a.0.y));

        return Some(Vec2::new(intersection_x, intersection_y));
    }

    None
}

pub fn line_rectangle_collision(line: Line2D, rect: Rect) -> Option<Vec2> {
    let left = line_line_collision(line, Line2D(rect.min, Vec2::new(rect.min.x, rect.max.y)));
    let top = line_line_collision(line, Line2D(Vec2::new(rect.min.x, rect.max.y), rect.max));

    let right = line_line_collision(line, Line2D(rect.max, Vec2::new(rect.max.x, rect.min.y)));
    let bottom = line_line_collision(
        line,
        Line2D(
            Vec2::new(rect.min.x, rect.min.y),
            Vec2::new(rect.max.x, rect.min.y),
        ),
    );

    if left.is_some() {
        return left;
    }

    if right.is_some() {
        return right;
    }

    if top.is_some() {
        return top;
    }

    if bottom.is_some() {
        return bottom;
    }

    None
}

pub fn rect_rect_collision(a: Rect, b: Rect) -> bool {
    let r1x = a.min.x;
    let r1w = a.width();
    let r1y = a.min.y;
    let r1h = a.height();

    let r2x = b.min.x;
    let r2w = b.width();
    let r2y = b.min.y;
    let r2h = b.height();

    if r1x + r1w > r2x &&    // r1 right edge past r2 left
        r1x < r2x + r2w &&    // r1 left edge past r2 right
        r1y + r1h > r2y &&    // r1 top edge past r2 bottom
        r1y < r2y + r2h
    {
        // r1 bottom edge past r2 top
        return true;
    }

    false
}
