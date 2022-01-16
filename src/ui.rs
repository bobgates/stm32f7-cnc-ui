use rtt_target::rprintln;

const MAX_WHOLE_NUMS: u8 = 3;
const N_DECIMALS: u8 = 3;

#[derive(Copy, Clone, Debug)]
pub struct State {
    ui: UIMode,
    entry: Option<i32>,
    x: i32,
    y: i32,
    z: i32,
    running: Running,
    error: bool,
    machine: MachineMode,
    // view: &'static view::View,
}

// use crate::view;

impl State {
    pub fn new() -> State {
        State {
            ui: UIMode::Resting,
            entry: None,
            x: 0,
            y: 0,
            z: 0,
            running: Running::No,
            error: false,
            machine: MachineMode::Absolute,
        }
    }

    pub fn event_loop(self, msg: Ids) {
        rprintln!("In event_loop: {:?}", msg);
        if msg == Ids::Y0Button {}
    }
}

#[derive(Copy, Clone, Debug)]
enum MachineMode {
    Absolute,
    Relative,
}

#[derive(Copy, Clone, Debug)]
enum UIMode {
    Resting,
    NumberEntry,
}

#[derive(Copy, Clone, Debug)]
enum Running {
    Yes,
    No,
    Jog,
}
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Ids {
    Key(u8),
    Clear,
    DecimalPoint,
    Enter,
    Half,
    PlusMinus,
    XButton,
    X0Button,
    YButton,
    Y0Button,
    ZButton,
    Z0Button,
    X(u32),
    Y(u32),
    Z(u32),
    Working(u32),
    Empty,
}

pub struct Update {}

impl Update {
    pub fn new() -> Update {
        Update {}
    }
}

// Update

// Create a view of the State and keypad, etc:
// View
