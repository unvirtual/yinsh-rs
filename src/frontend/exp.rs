use std::collections::HashMap;

use macroquad::prelude::{is_mouse_button_pressed, mouse_position, MouseButton};

use crate::common::coord::HexCoord;


#[derive(Eq, PartialEq)]
enum Event {
    MouseEntered,
    MouseLeft,
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum UserAction {
    AtCoord(HexCoord),
    Undo(),
}

enum ShapeState {
    Normal,
    Hoverable,
}
struct BBox {

}

impl BBox {
    fn point_within(&self, pt: (f32, f32)) -> bool {
        todo!();
    }
}


struct Shape {
    pos: (f32, f32),
    radius: f32,
}

impl Shape {
    fn render(&self) {}
    fn update(&mut self, event: Event) {
        //
    }
    fn point_within(&self, pt: (f32, f32)) -> bool {
        todo!();
    }
    fn coord(&self) -> HexCoord {
        todo!();
    }
}

type ShapeId = usize;

struct Window {
    shapes: HashMap<ShapeId, Shape>,
    mouse_pos: (f32, f32),
    last_mouse_pos: (f32, f32),
    last_user_action: Option<UserAction>,
    run_bboxes: HashMap<BBox, Vec<ShapeId>>,
    mouse_pointer: Shape,
}

impl Window {
    fn update(&mut self /*Game state */) {
        // only called if game state has changed
        //
        // update shapes
        // set shape properties
        // ...
    }

    fn render(&mut self) {
        // called every frame
        self.last_mouse_pos = self.mouse_pos;
        self.mouse_pos = mouse_position();

        self.handle_shape_events();
        self.handle_bbox_events();

        self.shapes.iter().map(|(_, s)| s.render());
        self.handle_mouse_clicked_action();
    }

    fn handle_shape_events(&mut self) {
        self.shapes.iter().map(|(id, s)| {
            if let Some(event) = self.event_for_shape_id(*id) {
                s.update(event);
            }
        });
    }

    fn handle_bbox_events(&mut self) {
        self.run_bboxes.iter().for_each(|(bbox, shape_ids)| {
            if let Some(event) = self.event_for_bbox(bbox) {
                shape_ids.iter().map(|id|
                    self.shapes[id].update(event)
                );
            }
        });
    }

    fn handle_mouse_clicked_action(&self) {
        let coord = HexCoord::new(0,0);
        self.last_user_action = Some(UserAction::AtCoord(coord));
    }

    fn poll_user_action(&self) -> Option<UserAction> {
        self.last_user_action.take()
    }

    fn event_if<F>(&self, f: F) -> Option<Event>
    where F: Fn((f32, f32)) -> bool {
        let within_now = f(self.mouse_pos);
        let within_before = f(self.last_mouse_pos);
        if within_now && !within_before {
            return Some(Event::MouseEntered);
        }
        if !within_now && within_before {
            return Some(Event::MouseLeft);
        }
        None
    }

    fn event_for_shape_id(&self, s: ShapeId) -> Option<Event> {
        self.event_if(|pos| self.shapes[&s].point_within(pos))
    }

    fn event_for_bbox(&self, bbox: &BBox) -> Option<Event> {
        self.event_if(|pos| bbox.point_within(pos))
    }
}
