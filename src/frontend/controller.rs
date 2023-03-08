use std::collections::HashMap;

use crate::{common::coord::Point, core::game::UserAction};

use super::{
    element::Element,
    mouse::{MouseEvent, MouseHandler},
    primitives::Message,
};

pub type ElementId = usize;

pub struct Controller {
    elements: HashMap<ElementId, Box<dyn Element>>,
    messages: HashMap<ElementId, Vec<Message>>,
    subscribers: HashMap<ElementId, Vec<ElementId>>,
    actions: Vec<UserAction>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            messages: HashMap::new(),
            subscribers: HashMap::new(),
            actions: vec![],
        }
    }

    pub fn add_element(&mut self, element: Box<dyn Element>) -> ElementId {
        let id = self.add_element_inactive(element);
        self.add_subscriber(id, id);
        id
    }

    pub fn add_element_inactive(&mut self, element: Box<dyn Element>) -> ElementId {
        let id = self.elements.len();
        self.elements.insert(id, element);
        id
    }

    pub fn get_actions(&self) -> Vec<UserAction> {
        self.actions.clone()
    }

    pub fn clear_all(&mut self) {
        self.elements.clear();
        self.messages.clear();
        self.subscribers.clear();
    }

    pub fn get_mouse_clicks(&self) -> Vec<Message> {
        self.messages
            .values()
            .flatten()
            .filter(|msg| {
                if let Message::MouseClicked(_) = msg {
                    true
                } else {
                    false
                }
            })
            .cloned()
            .collect::<Vec<_>>()
    }
        
    pub fn handle_input(&mut self, mouse_event: &MouseEvent) {
        self.mouse_input(mouse_event);
    }

    pub fn render(&mut self) {
        self.actions.clear();
        self.update_elements();
        self.render_elements();
    }

    fn render_elements(&self) {
        let mut sorted_elements: Vec<&Box<dyn Element>> = self.elements.values().collect();
        sorted_elements.sort_by(|a, b| a.z_value().cmp(&b.z_value()));
        sorted_elements.iter().for_each(|e| e.render());
    }

    fn mouse_input(&mut self, mouse_event: &MouseEvent) {
        for (id, element) in &self.elements {
            element
                .handle_events(mouse_event)
                .into_iter()
                .for_each(|msg| {
                    self.messages.entry(*id).or_default();
                    self.messages.get_mut(id).unwrap().push(msg);
                });
        }
    }

    fn update_elements(&mut self) {
        for (id, msg) in self.messages.drain() {
            self.subscribers.get(&id).map(|x| {
                x.iter().for_each(|sid| {
                    msg.iter().for_each(|m| {
                        let action = self.elements.get_mut(&sid).unwrap().update(&m);
                        action.map(|a| self.actions.push(a));
                    });
                })
            });
        }
    }

    pub fn add_subscriber(&mut self, source: ElementId, subscriber: ElementId) {
        self.subscribers.entry(source).or_default();
        self.subscribers.get_mut(&source).unwrap().push(subscriber);
    }
}
