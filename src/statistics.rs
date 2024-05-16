use serde::Serialize;

use crate::parse::Map;

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomDetail {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub tiles: String,
}

pub fn bounding_box(map: &Map) -> Option<BoundingBox> {
    let rooms = map.root.get_child("levels")?;

    let mut bounds: Option<(i64, i64, i64, i64)> = None;
    for room in &rooms.children {
        let x = room.get_attribute("x")?.as_integer()?;
        let y = room.get_attribute("y")?.as_integer()?;
        let width = room.get_attribute("width")?.as_integer()?;
        let height = room.get_attribute("height")?.as_integer()?;
        if let Some(bounds) = &mut bounds {
            *bounds = (
                bounds.0.min(x),
                bounds.1.min(y),
                bounds.2.max(x + width),
                bounds.3.max(y + height),
            );
        } else {
            bounds = Some((x, y, x + width, y + height));
        }
    }
    let bounds = bounds?;

    Some(BoundingBox {
        x: bounds.0,
        y: bounds.1,
        width: bounds.2 - bounds.0,
        height: bounds.3 - bounds.1,
    })
}

pub fn room_details(map: &Map) -> Option<Vec<RoomDetail>> {
    let rooms = map.root.get_child("levels")?;

    let mut room_details = vec![];

    for room in &rooms.children {
        // 360187 gives unnamed as the room.name
        // if room.name != "level" {
        //     return None;
        // }

        let x = room.get_attribute("x")?.as_integer()?;
        let y = room.get_attribute("y")?.as_integer()?;
        let width = room.get_attribute("width")?.as_integer()?;
        let height = room.get_attribute("height")?.as_integer()?;
        let solids = room.get_child("solids")?;
        let tiles = solids.get_attribute("innerText")?.as_string()?.clone();

        room_details.push(RoomDetail {
            x,
            y,
            width,
            height,
            tiles,
        });
    }

    Some(room_details)
}
