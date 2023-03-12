use std::collections::HashMap;
use std::hash::Hash;

use crate::{common::coord::Point, core::game::UiAction};

use super::{
    element::Element,
    mouse::{self, MouseEvent, MouseHandler},
    primitives::{Event, Message},
};

pub type ElementId = usize;

pub struct Controller {
    elements: HashMap<ElementId, Box<dyn Element>>,
    messages: HashMap<ElementId, Vec<Message>>,
    subscribers: HashMap<ElementId, Vec<ElementId>>,
    actions: Vec<UiAction>,
    events: Vec<Event>,
}

fn insert_hashmap_vec<K, V>(hashmap: &mut HashMap<K, Vec<V>>, key: K, value: V)
where
    K: Eq + Hash + Clone,
{
    hashmap.entry(key.clone()).or_default();
    hashmap.get_mut(&key).unwrap().push(value);
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
        // make sure that MouseEntered and MouseClicked Events only trigger Messages for elements with highest z-value.
        let mut max_z = -100;
        let mut mouse_entered_candidates = vec![];
        let mut mouse_clicked_candidates = vec![];

        for e in self.events.drain(0..) {
            self.elements.iter().for_each(|(id, element)| {
                element.handle_event(&e).into_iter().for_each(|msg| {
                    match msg {
                        Message::MouseInside | Message::MouseEntered => {
                            mouse_entered_candidates.push((*id, element.z_value()));
                            max_z = max_z.max(element.z_value());
                        }
                        msg @ Message::MouseClicked(_) => {
                            mouse_clicked_candidates.push((*id, element.z_value(), msg));
                            max_z = max_z.max(element.z_value());
                        }
                        _ => {
                            insert_hashmap_vec(&mut self.messages, *id, msg);
                        }
                    };
                });
            });
        }

        mouse_entered_candidates.into_iter().for_each(|(id, z)| {
            let msg = if z == max_z {
                Message::MouseEntered
            } else {
                Message::MouseLeft
            };
            insert_hashmap_vec(&mut self.messages, id, msg);
        });

        mouse_clicked_candidates.into_iter().for_each(|(id, z, msg)| {
            if z == max_z {
                insert_hashmap_vec(&mut self.messages, id, msg);
            }
        });
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
            self.subscribers.get(&id).map(|subscriber| {
                subscriber.iter().for_each(|subscriber_id| {
                    msg.iter().for_each(|m| {
                        let action = self.elements.get_mut(&subscriber_id).unwrap().update(&m);
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
