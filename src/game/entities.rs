#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn other(&self) -> Player {
        match *self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Piece {
    Ring(Player),
    Marker(Player),
}

impl Piece {
    pub fn is_ring(&self) -> bool {
        match *self {
            Piece::Ring(_) => true,
            _ => false,
        }
    }

    pub fn is_marker(&self) -> bool {
        match *self {
            Piece::Marker(_) => true,
            _ => false,
        }
    }

    pub fn belongs_to(&self, player: Player) -> bool {
        match self {
            Piece::Ring(p) => *p == player,
            Piece::Marker(p) => *p == player,
        }
    }

    pub fn owner(&self) -> Player {
        match self {
            Piece::Ring(p) => p.clone(),
            Piece::Marker(p) => p.clone(),
        }
    }

    pub fn flip(&self) -> Option<Piece> {
        match *self {
            Piece::Ring(p) => None,
            Piece::Marker(p) => Some(Piece::Marker(p.other())),
        }
    }
}
