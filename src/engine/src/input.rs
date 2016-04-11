#[derive(Debug)]
struct Key {
    key: String,
    // Modifiers
    shift: bool,
    control: bool,
    alt: bool,
}

#[derive(Debug)]
struct KeyboardBinding {
    main: Key,
    alt: Option<Key>,
}

#[derive(Debug)]
struct GamepadBinding {
    // TODO Allow for controller specific bindings
    //id: Option<u8>,
    main: String,
}

#[derive(Debug)]
struct InputBinding {
    action: String,
    keyboard: Option<KeyboardBinding>,
    gamepad: Option<GamepadBinding>,
}

// TODO Implement From for InputBinding

pub type InputBinds = Vec<InputBinding>;
