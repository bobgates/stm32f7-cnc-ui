const MAX_WHOLE_NUMS: u8 = 3;
const N_DECIMALS: u8 = 3;

struct State {
    entry: i32,
    x: i32,
    y: i32,
    z: i32,
    running: Running,
    error: bool,
    mode: Mode,
}

enum Mode {
    Absolute,
    Relative,
}

enum Running {
    Yes,
    No,
    Jog,
}

enum Messages {
    Key(u8),
    TouchPadClear,
    DecimalPoint,
    X(u32),
    Y(u32),
    Z(u32),
}

// Update

// Create a view of the State and keypad, etc:
// View
