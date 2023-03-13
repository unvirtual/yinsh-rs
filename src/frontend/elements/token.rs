use std::path::Ancestors;

use macroquad::prelude::*;

use crate::{
    common::coord::{distance_squared, HexCoord, Point},
    core::{entities::Player, game::UiAction},
    frontend::{
        animation::*,
        element::{Element, ShapeState},
        events::{Event, Message},
        mouse::mouse_leave_enter_event,
        primitives::draw_ring_mesh,
    },
};

use super::animated_token::AnimatedToken;

fn player_color(player: Player) -> Color {
    match player {
        Player::White => WHITE,
        Player::Black => BLACK,
    }
}

pub struct TokenConfig {
    pub ring_inner_radius: f32,
    pub ring_outer_radius: f32,
    pub marker_radius: f32,
    pub white_player_color: Color,
    pub black_player_color: Color,
    pub default_hover_color: Color,
    pub remove_hover_color: Color,
}

impl TokenConfig {
    pub fn new() -> Self {
        Self {
            ring_inner_radius: 0.2,
            ring_outer_radius: 0.5,
            marker_radius: 0.2,
            white_player_color: WHITE,
            black_player_color: BLACK,
            default_hover_color: BLUE,
            remove_hover_color: RED,
        }
    }
}

#[derive(Clone)]
pub enum TokenType {
    Ring(f32, f32),
    Marker(f32),
}

pub struct TokenBuilder {
    pos: Point,
    coord: Option<HexCoord>,
    pub token_type: Option<TokenType>,
    default_color: Option<Color>,
    hover_color: Option<Color>,
    state: Option<ShapeState>,
    z_value: Option<i32>,
    config: TokenConfig,
}

impl TokenBuilder {
    pub fn new() -> Self {
        let config = TokenConfig::new();
        Self {
            pos: Point(0., 0.),
            coord: None,
            token_type: None,
            default_color: None,
            hover_color: Some(config.default_hover_color),
            state: Some(ShapeState::Visible),
            z_value: Some(0),
            config,
        }
    }

    pub fn ring(&mut self, player: Player) -> &mut Self {
        self.token_type = Some(TokenType::Ring(
            self.config.ring_outer_radius,
            self.config.ring_inner_radius,
        ));
        self.set_player(player);
        self
    }

    pub fn marker(&mut self, player: Player) -> &mut Self {
        self.token_type = Some(TokenType::Marker(self.config.marker_radius));
        self.set_player(player);
        self
    }

    fn set_player(&mut self, player: Player) {
        match player {
            Player::Black => {
                self.default_color = Some(self.config.black_player_color);
            }
            Player::White => {
                self.default_color = Some(self.config.white_player_color);
            }
        }
    }

    pub fn remove_hover_color(&mut self) -> &mut Self {
        self.hover_color = Some(self.config.remove_hover_color);
        self
    }

    pub fn state(&mut self, state: ShapeState) -> &mut Self {
        self.state = Some(state);
        self
    }

    pub fn pos(&mut self, pos: Point) -> &mut Self {
        self.pos = pos;
        self
    }

    pub fn coord(&mut self, coord: HexCoord) -> &mut Self {
        self.coord = Some(coord);
        self.pos = Point::from(coord);
        self
    }

    pub fn z_value(&mut self, z_value: i32) -> &mut Self {
        self.z_value = Some(z_value);
        self
    }

    pub fn animate(&mut self, animation: Box<dyn Animation>) -> AnimatedToken {
        AnimatedToken::new(self.build(), animation)
    }

    pub fn build(&mut self) -> Token {
        Token {
            pos: self.pos,
            coord: self.coord,
            shape_type: self.token_type.clone().unwrap(),
            color: self.default_color.unwrap(),
            default_color: self.default_color.unwrap(),
            hover_color: self.hover_color.unwrap(),
            state: self.state.unwrap(),
            z_value: self.z_value.unwrap(),
            mouse_entered: false,
        }
    }
}


pub struct Token {
    pos: Point,
    coord: Option<HexCoord>,
    pub shape_type: TokenType,
    color: Color,
    default_color: Color,
    hover_color: Color,
    state: ShapeState,
    z_value: i32,
    mouse_entered: bool,
}

impl Token {
    pub fn new(
        pos: Point,
        coord: Option<HexCoord>,
        shape_type: TokenType,
        color: Color,
        z_value: i32,
    ) -> Self {
        Token {
            pos,
            coord,
            shape_type,
            color,
            default_color: color,
            hover_color: BLUE,
            state: ShapeState::Visible,
            z_value,
            mouse_entered: false,
        }
    }

    pub fn new_marker_at_coord(coord: HexCoord, player: Player, z_value: i32) -> Self {
        let pos = Point::from(coord);
        let mut elem = Token::new_marker_at_point(pos, player, z_value);
        elem.coord = Some(coord);
        elem
    }

    pub fn new_marker_at_point(pos: Point, player: Player, z_value: i32) -> Self {
        Token::new(
            pos,
            None,
            TokenType::Marker(0.2),
            player_color(player),
            z_value,
        )
    }

    pub fn new_ring_at_coord(coord: HexCoord, player: Player, z_value: i32) -> Self {
        let pos = Point::from(coord);
        let mut elem = Token::new_ring_at_point(pos, player, z_value);
        elem.coord = Some(coord);
        elem
    }

    pub fn new_ring_at_point(pos: Point, player: Player, z_value: i32) -> Self {
        Token::new(
            pos,
            None,
            TokenType::Ring(0.4, 0.2),
            player_color(player),
            z_value,
        )
    }

    pub fn draw(&self, color: Color) {
        match self.shape_type {
            TokenType::Ring(radius_outer, radius_inner) => {
                draw_circle_lines(self.pos.0, self.pos.1, radius_outer, 0.03, BLACK);
                draw_circle_lines(self.pos.0, self.pos.1, radius_inner, 0.03, BLACK);
                draw_ring_mesh(self.pos.0, self.pos.1, radius_inner, radius_outer, color);
            }
            TokenType::Marker(radius) => {
                draw_circle(self.pos.0, self.pos.1, radius, color);
                draw_circle_lines(self.pos.0, self.pos.1, radius, 0.03, BLACK);
            }
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn contains(&self, pos: Point) -> bool {
        match self.shape_type {
            TokenType::Marker(radius) => distance_squared(&self.pos, &pos) <= radius.powi(2),
            TokenType::Ring(outer, _) => distance_squared(&self.pos, &pos) <= outer.powi(2),
        }
    }

    pub fn pos(&self) -> Point {
        self.pos
    }

    pub fn set_pos(&mut self, pos: Point) {
        self.pos = pos;
    }

    pub fn coord(&self) -> Option<HexCoord> {
        self.coord
    }
}

impl Element for Token {
    fn render(&self) {
        if self.state == ShapeState::Invisible {
            return;
        }
        if self.state == ShapeState::Selected {
            self.draw(BLUE);
        } else {
            self.draw(self.color);
        }
    }

    fn update(&mut self, event: &Message) -> Option<UiAction> {
        match event {
            Message::MouseEntered => {
                self.color = self.hover_color;
                self.mouse_entered = true;
            }
            Message::MouseLeft => self.color = self.default_color,
            Message::ElementMoved(pt) => self.pos = *pt,
            _ => (),
        }
        None
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        match event {
            Event::Mouse(mouse_event) => {
                if self.state == ShapeState::Hoverable {
                    if let Some(e) = mouse_leave_enter_event(mouse_event, |pt| self.contains(*pt)) {
                        res.push(e);
                        return res;
                    };
                    if self.contains(mouse_event.pos) {
                        res.push(Message::MouseInside);
                    }
                }
                if self.state == ShapeState::AtMousePointer {
                    let pos = mouse_event
                        .legal_move_coord
                        .map(Point::from)
                        .unwrap_or(mouse_event.pos);
                    res.push(Message::ElementMoved(pos));
                }
            }
            _ => (),
        }
        res
    }

    fn set_state(&mut self, state: ShapeState) {
        self.state = state;
        match self.state {
            ShapeState::AtMousePointer => {
                self.color = Color::from_vec(self.color.to_vec() - vec4(0., 0., 0., 0.5));
            }
            _ => (),
        }
    }

    fn z_value(&self) -> i32 {
        self.z_value
    }
}
