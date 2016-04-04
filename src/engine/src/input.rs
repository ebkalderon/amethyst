#[derive(Debug)]
struct KeyboardBinding {
    main: String,
    alt: Option<String>,
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
