use std::collections::HashMap;

use crate::{common::coord::Point, core::game::UiAction};

use super::{
    element::Element,
    mouse::{MouseEvent, MouseHandler},
    primitives::{Message, Event},
};

pub type ElementId = usize;

pub struct Controller {
    elements: HashMap<ElementId, Box<dyn Element>>,
    messages: HashMap<ElementId, Vec<Message>>,
    subscribers: HashMap<ElementId, Vec<ElementId>>,
    actions: Vec<UiAction>,
    events: Vec<Event>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            messages: HashMap::new(),
            subscribers: HashMap::new(),
            actions: vec![],
            events: vec![],
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

    pub fn get_actions(&self) -> Vec<UiAction> {
        self.actions.clone()
    }

    pub fn clear_all(&mut self) {
        self.elements.clear();
        self.messages.clear();
        self.subscribers.clear();
        self.actions.clear();
        self.events.clear();
    }

    pub fn schedule_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn handle_events(&mut self) {
        for e in self.events.drain(0..) {
            for (id, element) in &self.elements {
                element
                    .handle_event(&e)
                    .into_iter()
                    .for_each(|msg| {
                        self.messages.entry(*id).or_default();
                        self.messages.get_mut(id).unwrap().push(msg);
                    });
            }
        }
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
