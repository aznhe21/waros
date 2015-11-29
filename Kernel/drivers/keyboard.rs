#[derive(Clone)]
pub struct Modifier {
    pub left: bool,
    pub right: bool
}

impl Modifier {
    #[inline]
    pub const fn none() -> Modifier {
        Modifier {
            left: false,
            right: false
        }
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.left || self.right
    }
}

// TODO: bitfields
// Based on DOM Level 3 KeyboardEvent.getModifierState()
#[derive(Clone)]
pub struct Modifiers {
    pub alt: Modifier,
    pub alt_gr: Modifier,
    pub caps_lock: Modifier,
    pub control: Modifier,
    //pub fn: Modifier,
    //pub fn_lock: Modifier,
    //pub hyper: Modifier,
    pub meta: Modifier,
    pub num_lock: Modifier,
    pub os: Modifier,
    pub scroll_lock: Modifier,
    pub shift: Modifier,
    //pub super: Modifier,
    //pub symbol: Modifier,
    //pub symbol_lock: Modifier,
}

#[derive(Clone)]
pub struct KeyState {
    pub modifiers: Modifiers,
    pub code: u16
}

impl KeyState {
    #[inline]
    pub const fn new() -> KeyState {
        KeyState {
            modifiers: Modifiers {
                alt: Modifier::none(),
                alt_gr: Modifier::none(),
                caps_lock: Modifier::none(),
                control: Modifier::none(),
                //fn: Modifier::none(),
                //fn_lock: Modifier::none(),
                //hyper: Modifier::none(),
                meta: Modifier::none(),
                num_lock: Modifier::none(),
                os: Modifier::none(),
                scroll_lock: Modifier::none(),
                shift: Modifier::none(),
                //super: Modifier::none(),
                //symbol: Modifier::none(),
                //symbol_lock: Modifier::none(),
            },
            code: 0
        }
    }
}

pub enum Keyboard {
    Down(KeyState),
    Press(KeyState),
    Up(KeyState)
}

