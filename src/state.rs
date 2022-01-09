/// Holds the state of the machine

enum CoordState {
    NoEntry,
}

struct Coord {
    state: CoordState,
    value: f32,
    stored: f32,
}
