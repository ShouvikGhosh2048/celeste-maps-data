use crate::{
    parse::Map,
    statistics::{bounding_box, room_details},
};

use raylib::prelude::*;

pub fn view_map(map: &Map) {
    let bounds = bounding_box(map).unwrap();
    let room_details = room_details(map).unwrap();

    let mut camera = Camera2D {
        zoom: 1.0,
        ..Default::default()
    };
    // Initial camera postion and initial mouse position when drag starts.
    let mut drag_start: Option<([f32; 2], Vector2)> = None;

    let (mut rl, thread) = raylib::init()
        .size(1200, 900)
        .title("Map visualization")
        .build();
    while !rl.window_should_close() {
        // Update
        // References:
        // 1) raylib [core] example - 2d camera mouse zoom.
        // 2) https://github.com/deltaphc/raylib-rs/blob/master/samples/camera2D.rs
        if rl.is_mouse_button_down(MouseButton::MOUSE_RIGHT_BUTTON) {
            if let Some(position) = drag_start {
                let drag_offset = rl.get_mouse_position() - position.1;
                camera.target.x = position.0[0] - drag_offset.x / camera.zoom;
                camera.target.y = position.0[1] - drag_offset.y / camera.zoom;
            } else {
                drag_start = Some(([camera.target.x, camera.target.y], rl.get_mouse_position()));
            }
        } else if let Some(position) = drag_start {
            let drag_offset = rl.get_mouse_position() - position.1;
            camera.target.x = position.0[0] - drag_offset.x / camera.zoom;
            camera.target.y = position.0[1] - drag_offset.y / camera.zoom;
            drag_start = None;
        }

        let wheel_move = rl.get_mouse_wheel_move();
        if wheel_move.abs() > 1e-3 && drag_start.is_none() {
            let point_under_mouse = camera.target + rl.get_mouse_position() / camera.zoom;
            camera.zoom *= 1.1_f32.powf(wheel_move);
            camera.zoom = camera.zoom.max(0.01).min(10.0);
            camera.target = point_under_mouse - rl.get_mouse_position() / camera.zoom;
        }

        // Draw
        let mut d = rl.begin_drawing(&thread);
        let mut d_camera = d.begin_mode2D(camera);
        d_camera.clear_background(Color::WHITE);

        d_camera.draw_rectangle(
            bounds.x as i32,
            bounds.y as i32,
            bounds.width as i32,
            bounds.height as i32,
            Color::new(230, 230, 230, 255),
        );

        // Draw room background first incase of overlap.
        for room in &room_details {
            d_camera.draw_rectangle(
                room.x as i32,
                room.y as i32,
                room.width as i32,
                room.height as i32,
                Color::new(200, 200, 200, 255),
            );
        }

        for room in &room_details {
            let mut draw_x = room.x;
            let mut draw_y = room.y;
            for ch in room.tiles.chars() {
                if ch == '\n' {
                    draw_x = room.x;
                    draw_y += 8;
                    continue;
                }

                if ch != '0' {
                    d_camera.draw_rectangle(draw_x as i32, draw_y as i32, 8, 8, Color::BLACK);
                }
                draw_x += 8;
            }
        }
    }
}
