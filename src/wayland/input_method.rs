//! Native Wayland input-method backends.
//!
//! `zwp_input_method_v2` is preferred when available (COSMIC and newer
//! compositors advertise it through `zwp_input_method_manager_v2`). The older
//! `zwp_input_method_v1` backend remains available for compositors that expose
//! the older context/keyboard-grab model.

use crate::layout::{Composer, Layout};
use crate::wayland::xkb::Keymap;
use crate::wayland::{Connection, Decoder, Encoder, Event, Global};
use std::io;

const KEY_PRESSED: u32 = 1;

pub struct InputMethodService {
    layout: Layout,
}

impl InputMethodService {
    pub fn new(layout: Layout) -> Self {
        Self { layout }
    }

    pub fn run(self) -> io::Result<()> {
        let globals = crate::wayland::probe_globals()?;
        if globals.iter().any(|global| global.interface == "zwp_input_method_manager_v2") {
            return run_v2(self.layout);
        }
        if globals.iter().any(|global| global.interface == "zwp_input_method_v1") {
            return run_v1(self.layout);
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "compositor advertises neither zwp_input_method_v2 nor zwp_input_method_v1; run `unikey-rs doctor`",
        ))
    }
}

fn run_v1(layout: Layout) -> io::Result<()> {
    let mut connection = Connection::connect()?;
    let globals = connection.registry()?;
    let input_method = globals
        .iter()
        .find(|global| global.interface == "zwp_input_method_v1")
        .ok_or_else(|| missing("zwp_input_method_v1"))?;
    let input_method_id = connection.alloc_id();
    connection.bind(input_method, 1, input_method_id)?;

    let mut state = V1State {
        layout,
        composer: Composer::new(layout),
        serial: 0,
        context: None,
        keyboard: None,
        keymap: Keymap::default(),
        shift: false,
    };

    loop {
        let event = connection.recv()?;
        state.handle(&mut connection, input_method_id, event)?;
    }
}

struct V1State {
    layout: Layout,
    composer: Composer,
    serial: u32,
    context: Option<u32>,
    keyboard: Option<u32>,
    keymap: Keymap,
    shift: bool,
}

impl V1State {
    fn handle(&mut self, connection: &mut Connection, input_method_id: u32, event: Event) -> io::Result<()> {
        if event.object == input_method_id {
            match event.opcode {
                0 => {
                    let mut decoder = Decoder::new(&event.body);
                    self.context = Some(decoder.u32()?);
                    self.composer.clear();
                    let keyboard = connection.alloc_id();
                    self.keyboard = Some(keyboard);
                    connection.send(self.context.expect("context"), 9, Encoder::new().u32(keyboard))?;
                }
                1 => {
                    self.context = None;
                    self.keyboard = None;
                    self.composer.clear();
                }
                _ => {}
            }
            return Ok(());
        }

        if Some(event.object) == self.context {
            match event.opcode {
                1 => self.composer.clear(),
                4 => self.serial = Decoder::new(&event.body).u32()?,
                _ => {}
            }
            return Ok(());
        }

        if Some(event.object) == self.keyboard && event.opcode == 0 {
            self.keymap = load_keymap(event)?;
            return Ok(());
        }

        if Some(event.object) == self.keyboard && event.opcode == 4 {
            let mut decoder = Decoder::new(&event.body);
            let _serial = decoder.u32()?;
            let depressed = decoder.u32()?;
            let _latched = decoder.u32()?;
            let _locked = decoder.u32()?;
            let _group = decoder.u32()?;
            self.shift = depressed & 1 != 0;
            return Ok(());
        }

        if Some(event.object) == self.keyboard && event.opcode == 3 {
            let mut decoder = Decoder::new(&event.body);
            let _serial = decoder.u32()?;
            let _time = decoder.u32()?;
            let key = decoder.u32()?;
            let state = decoder.u32()?;
            if state == KEY_PRESSED {
                self.on_key(connection, key)?;
            }
        }
        Ok(())
    }

    fn on_key(&mut self, connection: &mut Connection, key: u32) -> io::Result<()> {
        let Some(context) = self.context else { return Ok(()) };
        match key {
            14 => {
                self.composer.backspace();
                self.preedit_v1(connection, context)?;
            }
            28 => self.commit_v1(connection, context, "\n")?,
            57 => self.commit_v1(connection, context, " ")?,
            _ => {
                if let Some(ch) = self.keymap.key_to_char(key, self.shift).or_else(|| key_to_char_with_shift(key, self.shift)) {
                    self.composer.push_char(ch);
                    if self.layout == Layout::English {
                        self.commit_v1(connection, context, &ch.to_string())?;
                    } else {
                        self.preedit_v1(connection, context)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn preedit_v1(&self, connection: &mut Connection, context: u32) -> io::Result<()> {
        connection.send(context, 2, Encoder::new().u32(self.serial).string(&self.composer.output()).string(""))
    }

    fn commit_v1(&mut self, connection: &mut Connection, context: u32, suffix: &str) -> io::Result<()> {
        let text = if self.layout == Layout::English {
            suffix.to_owned()
        } else {
            self.composer.push_str(suffix);
            let text = self.composer.output();
            self.composer.clear();
            text
        };
        connection.send(context, 1, Encoder::new().u32(self.serial).string(&text))
    }
}

fn run_v2(layout: Layout) -> io::Result<()> {
    let mut connection = Connection::connect()?;
    let globals = connection.registry()?;
    let manager = globals
        .iter()
        .find(|global| global.interface == "zwp_input_method_manager_v2")
        .ok_or_else(|| missing("zwp_input_method_manager_v2"))?;
    let seat = globals
        .iter()
        .find(|global| global.interface == "wl_seat")
        .ok_or_else(|| missing("wl_seat"))?;

    let manager_id = connection.alloc_id();
    connection.bind(manager, 1, manager_id)?;
    let seat_id = connection.alloc_id();
    connection.bind(seat, 1, seat_id)?;
    let input_method_id = connection.alloc_id();
    connection.send(manager_id, 0, Encoder::new().u32(seat_id).u32(input_method_id))?;

    let mut state = V2State {
        layout,
        composer: Composer::new(layout),
        serial: 0,
        grab: None,
        keymap: Keymap::default(),
        shift: false,
    };

    loop {
        let event = connection.recv()?;
        state.handle(&mut connection, input_method_id, event)?;
    }
}

struct V2State {
    layout: Layout,
    composer: Composer,
    serial: u32,
    grab: Option<u32>,
    keymap: Keymap,
    shift: bool,
}

impl V2State {
    fn handle(&mut self, connection: &mut Connection, input_method_id: u32, event: Event) -> io::Result<()> {
        if event.object == input_method_id {
            match event.opcode {
                0 => {
                    self.composer.clear();
                    if self.grab.is_none() {
                        let grab = connection.alloc_id();
                        self.grab = Some(grab);
                        connection.send(input_method_id, 5, Encoder::new().u32(grab))?;
                    }
                }
                1 => {
                    self.grab = None;
                    self.composer.clear();
                }
                5 => self.serial = self.serial.saturating_add(1),
                6 => {
                    return Err(io::Error::new(
                        io::ErrorKind::ConnectionAborted,
                        "input method became unavailable",
                    ))
                }
                _ => {}
            }
            return Ok(());
        }

        if Some(event.object) == self.grab {
            match event.opcode {
                0 => self.keymap = load_keymap(event)?,
                1 => {
                    let mut decoder = Decoder::new(&event.body);
                    let _serial = decoder.u32()?;
                    let _time = decoder.u32()?;
                    let key = decoder.u32()?;
                    let pressed = decoder.u32()? == KEY_PRESSED;
                    if pressed {
                        self.on_key(connection, input_method_id, key)?;
                    }
                }
                2 => {
                    let mut decoder = Decoder::new(&event.body);
                    let _serial = decoder.u32()?;
                    let depressed = decoder.u32()?;
                    let _latched = decoder.u32()?;
                    let _locked = decoder.u32()?;
                    let _group = decoder.u32()?;
                    self.shift = depressed & 1 != 0;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn on_key(&mut self, connection: &mut Connection, input_method_id: u32, key: u32) -> io::Result<()> {
        match key {
            14 => {
                self.composer.backspace();
                self.preedit_v2(connection, input_method_id)?;
            }
            28 => self.commit_v2(connection, input_method_id, "\n")?,
            57 => self.commit_v2(connection, input_method_id, " ")?,
            _ => {
                if let Some(ch) = self.keymap.key_to_char(key, self.shift).or_else(|| key_to_char_with_shift(key, self.shift)) {
                    self.composer.push_char(ch);
                    if self.layout == Layout::English {
                        self.commit_v2(connection, input_method_id, &ch.to_string())?;
                    } else {
                        self.preedit_v2(connection, input_method_id)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn preedit_v2(&self, connection: &mut Connection, input_method_id: u32) -> io::Result<()> {
        connection.send(
            input_method_id,
            1,
            Encoder::new().string(&self.composer.output()).i32(-1).i32(-1),
        )?;
        connection.send(input_method_id, 3, Encoder::new().u32(self.serial))
    }

    fn commit_v2(&mut self, connection: &mut Connection, input_method_id: u32, suffix: &str) -> io::Result<()> {
        let text = if self.layout == Layout::English {
            suffix.to_owned()
        } else {
            self.composer.push_str(suffix);
            let text = self.composer.output();
            self.composer.clear();
            text
        };
        connection.send(input_method_id, 0, Encoder::new().string(&text))?;
        connection.send(input_method_id, 1, Encoder::new().string("").i32(-1).i32(-1))?;
        connection.send(input_method_id, 3, Encoder::new().u32(self.serial))
    }
}

/// Load the compositor-provided XKB keymap. The small evdev/QWERTY map below
/// remains a fallback for `no_keymap` compositor test environments.
pub fn load_keymap(event: Event) -> io::Result<Keymap> {
    let Event { body, mut fds, .. } = event;
    let mut decoder = Decoder::new(&body);
    let format = decoder.u32()?;
    let size = decoder.u32()? as usize;
    if format != 0 {
        return Ok(Keymap::default());
    }
    match fds.pop() {
        Some(fd) => Keymap::from_fd(fd, size),
        None => Ok(Keymap::default()),
    }
}

pub fn key_to_char(key: u32) -> Option<char> {
    key_to_char_with_shift(key, false)
}

pub fn key_to_char_with_shift(key: u32, shift: bool) -> Option<char> {
    let ch = match key {
        2..=11 => [Some('1'), Some('2'), Some('3'), Some('4'), Some('5'), Some('6'), Some('7'), Some('8'), Some('9'), Some('0')][key as usize - 2],
        12 => Some('-'),
        13 => Some('='),
        16..=25 => [Some('q'), Some('w'), Some('e'), Some('r'), Some('t'), Some('y'), Some('u'), Some('i'), Some('o'), Some('p')][key as usize - 16],
        26 => Some('['),
        27 => Some(']'),
        28 => Some('\n'),
        30..=40 => [Some('a'), Some('s'), Some('d'), Some('f'), Some('g'), Some('h'), Some('j'), Some('k'), Some('l'), Some(';'), Some('\'')][key as usize - 30],
        41 => Some('`'),
        43 => Some('\\'),
        44..=53 => [Some('z'), Some('x'), Some('c'), Some('v'), Some('b'), Some('n'), Some('m'), Some(','), Some('.'), Some('/')][key as usize - 44],
        57 => Some(' '),
        _ => None,
    };
    let ch = ch?;
    if !shift {
        return Some(ch);
    }
    Some(match ch {
        '1' => '!', '2' => '@', '3' => '#', '4' => '$', '5' => '%',
        '6' => '^', '7' => '&', '8' => '*', '9' => '(', '0' => ')',
        '-' => '_', '=' => '+', '[' => '{', ']' => '}', '\\' => '|',
        '`' => '~', ';' => ':', '"' => '\'', ',' => '<', '.' => '>', '/' => '?',
        other => other.to_ascii_uppercase(),
    })
}

pub fn find_global<'a>(globals: &'a [Global], interface: &str) -> Option<&'a Global> {
    globals.iter().find(|global| global.interface == interface)
}

fn missing(interface: &str) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("Wayland global `{interface}` is unavailable"))
}
