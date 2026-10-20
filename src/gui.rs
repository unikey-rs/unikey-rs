//! Dependency-free Wayland GUI.
//!
//! The GUI is a native `xdg_shell` + `wl_shm` client. It intentionally uses a
//! tiny software canvas instead of GTK/Qt so the AppImage has no toolkit
//! runtime and the same layout engine can be exercised interactively.

use crate::layout::{Composer, Layout};
use crate::wayland::{Connection, Decoder, Encoder, Event};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::os::fd::AsFd;

const WIDTH: u32 = 820;
const HEIGHT: u32 = 540;
const BG: u32 = 0xfff4f1e8;
const INK: u32 = 0xff20252b;
const BLUE: u32 = 0xff2463a7;
const ORANGE: u32 = 0xffc7662c;
const GREEN: u32 = 0xff2e8b67;

pub fn run(initial: Layout) -> io::Result<()> {
    let mut connection = Connection::connect()?;
    let globals = connection.registry()?;
    let compositor = globals.iter().find(|g| g.interface == "wl_compositor").ok_or_else(|| missing("wl_compositor"))?;
    let shm = globals.iter().find(|g| g.interface == "wl_shm").ok_or_else(|| missing("wl_shm"))?;
    let wm_base = globals.iter().find(|g| g.interface == "xdg_wm_base").ok_or_else(|| missing("xdg_wm_base"))?;
    let seat = globals.iter().find(|g| g.interface == "wl_seat").ok_or_else(|| missing("wl_seat"))?;

    let compositor_id = connection.alloc_id();
    connection.bind(compositor, 4, compositor_id)?;
    let shm_id = connection.alloc_id();
    connection.bind(shm, 1, shm_id)?;
    let wm_base_id = connection.alloc_id();
    connection.bind(wm_base, 2, wm_base_id)?;
    let seat_id = connection.alloc_id();
    connection.bind(seat, 5, seat_id)?;

    let surface_id = connection.alloc_id();
    connection.send(compositor_id, 0, Encoder::new().u32(surface_id))?;
    let xdg_surface_id = connection.alloc_id();
    connection.send(wm_base_id, 2, Encoder::new().u32(xdg_surface_id).u32(surface_id))?;
    let toplevel_id = connection.alloc_id();
    connection.send(xdg_surface_id, 1, Encoder::new().u32(toplevel_id))?;
    connection.send(toplevel_id, 2, Encoder::new().string("UniKey RS"))?;
    connection.send(toplevel_id, 3, Encoder::new().string("org.unikey-rs.UniKeyRS"))?;
    connection.send(surface_id, 6, &Encoder::new())?;

    let pointer_id = connection.alloc_id();
    connection.send(seat_id, 0, Encoder::new().u32(pointer_id))?;
    let keyboard_id = connection.alloc_id();
    connection.send(seat_id, 1, Encoder::new().u32(keyboard_id))?;

    let mut state = GuiState {
        composer: Composer::new(initial),
        layout: initial,
        cursor_x: 0.0,
        cursor_y: 0.0,
        configured: false,
        running: true,
    };

    loop {
        let event = connection.recv()?;
        handle_event(
            &mut connection,
            event,
            &mut state,
            surface_id,
            xdg_surface_id,
            toplevel_id,
            pointer_id,
            keyboard_id,
            shm_id,
        )?;
        if !state.running {
            return Ok(());
        }
    }
}

struct GuiState {
    composer: Composer,
    layout: Layout,
    cursor_x: f64,
    cursor_y: f64,
    configured: bool,
    running: bool,
}

#[allow(clippy::too_many_arguments)]
fn handle_event(
    connection: &mut Connection,
    event: Event,
    state: &mut GuiState,
    surface_id: u32,
    xdg_surface_id: u32,
    toplevel_id: u32,
    pointer_id: u32,
    keyboard_id: u32,
    shm_id: u32,
) -> io::Result<()> {
    if event.object == xdg_surface_id && event.opcode == 0 {
        let mut decoder = Decoder::new(&event.body);
        let serial = decoder.u32()?;
        connection.send(xdg_surface_id, 4, Encoder::new().u32(serial))?;
        if !state.configured {
            state.configured = true;
            let pixels = draw(state);
            let buffer_id = create_buffer(connection, shm_id, &pixels)?;
            connection.send(surface_id, 1, Encoder::new().u32(buffer_id).i32(0).i32(0))?;
            connection.send(surface_id, 2, Encoder::new().i32(0).i32(0).i32(WIDTH as i32).i32(HEIGHT as i32))?;
            connection.send(surface_id, 6, &Encoder::new())?;
        }
        return Ok(());
    }

    if event.object == toplevel_id && event.opcode == 1 {
        state.running = false;
        return Ok(());
    }

    if event.object == pointer_id {
        match event.opcode {
            2 => {
                let mut decoder = Decoder::new(&event.body);
                let _time = decoder.u32()?;
                state.cursor_x = fixed_to_f64(decoder.i32()?);
                state.cursor_y = fixed_to_f64(decoder.i32()?);
            }
            3 => {
                let mut decoder = Decoder::new(&event.body);
                let _serial = decoder.u32()?;
                let _time = decoder.u32()?;
                let button = decoder.u32()?;
                let pressed = decoder.u32()? == 1;
                if pressed && button == 0x110 {
                    click(connection, state, surface_id, shm_id)?;
                }
            }
            _ => {}
        }
        return Ok(());
    }

    if event.object == keyboard_id && event.opcode == 3 {
        let mut decoder = Decoder::new(&event.body);
        let _serial = decoder.u32()?;
        let _time = decoder.u32()?;
        let key = decoder.u32()?;
        let pressed = decoder.u32()? == 1;
        if pressed {
            key_press(connection, state, key, surface_id, shm_id)?;
        }
    }

    Ok(())
}

fn click(connection: &mut Connection, state: &mut GuiState, surface_id: u32, shm_id: u32) -> io::Result<()> {
    let x = state.cursor_x;
    let y = state.cursor_y;
    if (150.0..260.0).contains(&y) {
        if (40.0..205.0).contains(&x) {
            state.layout = Layout::English;
        } else if (225.0..390.0).contains(&x) {
            state.layout = Layout::Telex;
        } else if (410.0..575.0).contains(&x) {
            state.layout = Layout::Vni;
        } else if (595.0..780.0).contains(&x) {
            state.layout = Layout::Viqr;
        }
        state.composer.set_layout(state.layout);
    } else if (430.0..495.0).contains(&y) && (650.0..780.0).contains(&x) {
        state.running = false;
        return Ok(());
    }
    redraw(connection, state, surface_id, shm_id)
}

fn key_press(connection: &mut Connection, state: &mut GuiState, key: u32, surface_id: u32, shm_id: u32) -> io::Result<()> {
    match key {
        1 => state.running = false,
        2 => state.layout = Layout::English,
        3 => state.layout = Layout::Telex,
        4 => state.layout = Layout::Vni,
        5 => state.layout = Layout::Viqr,
        14 => {
            state.composer.backspace();
        }
        _ => {
            if let Some(ch) = crate::wayland::input_method::key_to_char(key) {
                state.composer.push_char(ch);
            }
        }
    }
    state.composer.set_layout(state.layout);
    redraw(connection, state, surface_id, shm_id)
}

fn redraw(connection: &mut Connection, state: &GuiState, surface_id: u32, shm_id: u32) -> io::Result<()> {
    let pixels = draw(state);
    let buffer_id = create_buffer(connection, shm_id, &pixels)?;
    connection.send(surface_id, 1, Encoder::new().u32(buffer_id).i32(0).i32(0))?;
    connection.send(surface_id, 2, Encoder::new().i32(0).i32(0).i32(WIDTH as i32).i32(HEIGHT as i32))?;
    connection.send(surface_id, 6, &Encoder::new())
}

fn create_buffer(connection: &mut Connection, shm_id: u32, pixels: &[u8]) -> io::Result<u32> {
    let path = std::env::temp_dir().join(format!("unikey-rs-{}-{}.shm", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
    let mut file = OpenOptions::new().create(true).truncate(true).read(true).write(true).open(&path)?;
    file.set_len(pixels.len() as u64)?;
    file.write_all(pixels)?;
    file.flush()?;
    let pool_id = connection.alloc_id();
    let mut encoder = Encoder::new();
    encoder.u32(pool_id).u32(pixels.len() as u32);
    encoder.fd(file.as_fd());
    connection.send(shm_id, 0, &encoder)?;
    let buffer_id = connection.alloc_id();
    let mut encoder = Encoder::new();
    encoder
        .u32(buffer_id)
        .u32(0)
        .u32(WIDTH)
        .u32(HEIGHT)
        .u32(WIDTH * 4)
        .u32(0);
    connection.send(pool_id, 0, &encoder)?;
    Ok(buffer_id)
}

fn draw(state: &GuiState) -> Vec<u8> {
    let mut pixels = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
    fill(&mut pixels, 0, 0, WIDTH, HEIGHT, BG);
    fill(&mut pixels, 0, 0, WIDTH, 74, BLUE);
    text(&mut pixels, 34, 25, "UNIKEY RS", 30, 0xffffffff);
    text(&mut pixels, 620, 27, "WAYLAND NATIVE", 18, GREEN);

    text(&mut pixels, 40, 108, "COMPOSER LAYOUT", 19, INK);
    button(&mut pixels, 40, 150, 165, 72, "ENGLISH", state.layout == Layout::English);
    button(&mut pixels, 225, 150, 165, 72, "TELEX", state.layout == Layout::Telex);
    button(&mut pixels, 410, 150, 165, 72, "VNI", state.layout == Layout::Vni);
    button(&mut pixels, 595, 150, 185, 72, "VIQR", state.layout == Layout::Viqr);

    text(&mut pixels, 40, 265, "TYPE HERE", 19, INK);
    fill(&mut pixels, 40, 292, 740, 68, 0xffffffff);
    outline(&mut pixels, 40, 292, 740, 68, 0xffc9ced3);
    let output = state.composer.output();
    let typed = state.composer.typed();
    text(&mut pixels, 58, 307, &format!("RAW  {}", tail(typed, 42)), 17, 0xff69727b);
    text(&mut pixels, 58, 334, &format!("OUT  {}", tail(&output, 42)), 19, INK);

    text(&mut pixels, 40, 405, "Click a layout or press 1-4. Backspace edits the preview.", 17, 0xff69727b);
    button(&mut pixels, 650, 430, 130, 65, "QUIT", false);
    pixels
}

fn button(pixels: &mut [u8], x: u32, y: u32, w: u32, h: u32, label: &str, active: bool) {
    let color = if active { ORANGE } else { 0xffffffff };
    fill(pixels, x, y, w, h, color);
    outline(pixels, x, y, w, h, if active { ORANGE } else { 0xffaeb7bf });
    text(pixels, x + 18, y + 24, label, 18, if active { 0xffffffff } else { INK });
}

fn fill(pixels: &mut [u8], x: u32, y: u32, w: u32, h: u32, color: u32) {
    for py in y..(y + h).min(HEIGHT) {
        for px in x..(x + w).min(WIDTH) {
            put_pixel(pixels, px, py, color);
        }
    }
}

fn outline(pixels: &mut [u8], x: u32, y: u32, w: u32, h: u32, color: u32) {
    fill(pixels, x, y, w, 1, color);
    fill(pixels, x, y + h - 1, w, 1, color);
    fill(pixels, x, y, 1, h, color);
    fill(pixels, x + w - 1, y, 1, h, color);
}

fn put_pixel(pixels: &mut [u8], x: u32, y: u32, color: u32) {
    let index = ((y * WIDTH + x) * 4) as usize;
    if index + 3 < pixels.len() {
        pixels[index] = color as u8;
        pixels[index + 1] = (color >> 8) as u8;
        pixels[index + 2] = (color >> 16) as u8;
        pixels[index + 3] = (color >> 24) as u8;
    }
}

fn text(pixels: &mut [u8], x: u32, y: u32, value: &str, scale: u32, color: u32) {
    let mut cursor = x;
    for ch in value.chars() {
        let glyph = glyph(ch);
        for row in 0..7u32 {
            for col in 0..5u32 {
                if glyph[row as usize] & (1 << (4 - col)) != 0 {
                    fill(pixels, cursor + col * scale / 2, y + row * scale / 2, scale.div_ceil(2), scale.div_ceil(2), color);
                }
            }
        }
        cursor += 4 * scale / 2;
    }
}

fn tail(value: &str, max: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= max {
        value.to_owned()
    } else {
        chars[chars.len() - max..].iter().collect()
    }
}

fn fixed_to_f64(value: i32) -> f64 {
    value as f64 / 256.0
}

fn missing(interface: &str) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("Wayland global `{interface}` is unavailable"))
}

/// 5x7 bitmap font. Unknown characters are rendered as a small box.
fn glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110],
        '6' => [0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '_' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00110],
        ':' => [0b00000, 0b00110, 0b00110, 0b00000, 0b00110, 0b00110, 0b00000],
        '?' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        ' ' => [0; 7],
        _ => [0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111],
    }
}
