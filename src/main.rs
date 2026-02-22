#![no_std]
#![no_main]

mod carapace;
mod chest;
mod claw;
mod composite_screen;
mod ferriscript;
mod load_euro_gylph;
mod open_new_window;
mod play_ferris_maker;
mod play_ferrisdungeon;
mod play_ferrisjump;
mod startup;

use carapace::*;
use chest::*;
use claw::*;
use composite_screen::*;
use core::arch::asm;
use core::panic::PanicInfo;
use ferriscript::*;
use load_euro_gylph::*;
use open_new_window::*;
use play_ferris_maker::*;
use play_ferrisdungeon::*;
use play_ferrisjump::*;
use startup::*;

#[derive(Copy, Clone, PartialEq)]
pub struct File {
    pub name: [u8; 16],
    pub content: [u8; 512],
    pub folder_id: u8,
    pub active: bool,
}

pub static mut FILES: [File; 10] = [File {
    name: [0; 16],
    content: [0; 512],
    folder_id: 0,
    active: false,
}; 10];

pub static mut WIN_DIR: [u8; 3] = [255, 255, 255];

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_ADDRESS: *mut u8 = 0xb8000 as *mut u8;

static mut BACKBUFFER: [u16; 2000] = [0; 2000];

static mut MOUSE_X: i32 = 40;
static mut MOUSE_Y: i32 = 12;

#[derive(Copy, Clone)]
struct Window {
    active: bool,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    cx: usize,
    cy: usize,
    buf: [u8; 128],
    buf_len: usize,
    screen: [u16; 50 * 15],
}

static mut WINDOWS: [Window; 3] = [Window {
    active: false,
    x: 0,
    y: 0,
    w: 46,
    h: 14,
    cx: 0,
    cy: 0,
    buf: [0; 128],
    buf_len: 0,
    screen: [0; 50 * 15],
}; 3];

static mut FOCUSED_WIN: usize = 0;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    asm!("in al, dx", out("al") result, in("dx") port, options(nomem, nostack, preserves_flags));
    result
}

unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

pub unsafe fn outw(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
}

unsafe fn play_sound(freq: u32) {
    if freq == 0 {
        return;
    }
    let div = 1193180 / freq;
    outb(0x43, 0xB6);
    outb(0x42, (div & 0xFF) as u8);
    outb(0x42, (div >> 8) as u8);
    let tmp = inb(0x61);
    if (tmp & 3) != 3 {
        outb(0x61, tmp | 3);
    }
}

unsafe fn stop_sound() {
    let tmp = inb(0x61) & 0xFC;
    outb(0x61, tmp);
}

unsafe fn init_mouse() {
    while (inb(0x64) & 2) != 0 {}
    outb(0x64, 0xA8);

    while (inb(0x64) & 2) != 0 {}
    outb(0x64, 0x20);
    while (inb(0x64) & 1) == 0 {}
    let status = inb(0x60) | 2;
    while (inb(0x64) & 2) != 0 {}
    outb(0x64, 0x60);
    while (inb(0x64) & 2) != 0 {}
    outb(0x60, status);

    while (inb(0x64) & 2) != 0 {}
    outb(0x64, 0xD4);
    while (inb(0x64) & 2) != 0 {}
    outb(0x60, 0xF6);
    while (inb(0x64) & 1) == 0 {}
    inb(0x60);

    while (inb(0x64) & 2) != 0 {}
    outb(0x64, 0xD4);
    while (inb(0x64) & 2) != 0 {}
    outb(0x60, 0xF4);
    while (inb(0x64) & 1) == 0 {}
    inb(0x60);
}

unsafe fn poll_mouse() -> Option<(i32, i32, bool, bool)> {
    let status = inb(0x64);
    if (status & 1) != 0 && (status & 0x20) != 0 {
        let flags = inb(0x60);
        if (flags & 0x08) != 0 {
            let mut t = 10000;
            while (inb(0x64) & 1) == 0 && t > 0 {
                t -= 1;
            }
            let dx = inb(0x60) as i8 as i32;

            let mut t = 10000;
            while (inb(0x64) & 1) == 0 && t > 0 {
                t -= 1;
            }
            let dy = inb(0x60) as i8 as i32;

            return Some((dx, dy, (flags & 1) != 0, (flags & 2) != 0));
        }
    }
    None
}

unsafe fn poll_keyboard() -> Option<u8> {
    let status = inb(0x64);
    if (status & 1) != 0 && (status & 0x20) == 0 {
        return Some(inb(0x60));
    }
    None
}

fn sleep_ms(ms: u32) {
    for _ in 0..ms {
        unsafe {
            outb(0x43, 0x30);
            outb(0x40, 0xA9);
            outb(0x40, 0x04);

            let mut prev_count = 0xFFFF;

            loop {
                outb(0x43, 0x00);
                let lo = inb(0x40);
                let hi = inb(0x40);
                let count = ((hi as u16) << 8) | (lo as u16);

                if count == 0 || count > prev_count {
                    break;
                }

                prev_count = count;
            }
        }
    }
}

fn scancode_to_char(scancode: u8, shift: bool) -> Option<u8> {
    match scancode {
        0x02 => Some(if shift { b'!' } else { b'1' }),
        0x03 => Some(if shift { b'@' } else { b'2' }),
        0x04 => Some(if shift { b'#' } else { b'3' }),
        0x05 => Some(if shift { b'$' } else { b'4' }),
        0x06 => Some(if shift { b'%' } else { b'5' }),
        0x07 => Some(if shift { b'^' } else { b'6' }),
        0x08 => Some(if shift { b'&' } else { b'7' }),
        0x09 => Some(if shift { b'*' } else { b'8' }),
        0x0A => Some(if shift { b'(' } else { b'9' }),
        0x0B => Some(if shift { b')' } else { b'0' }),
        0x0C => Some(if shift { b'_' } else { b'-' }),
        0x0D => Some(if shift { b'+' } else { b'=' }),
        0x0E => Some(0x08),
        0x0F => Some(b'\t'),
        0x10 => Some(if shift { b'Q' } else { b'q' }),
        0x11 => Some(if shift { b'W' } else { b'w' }),
        0x12 => Some(if shift { b'E' } else { b'e' }),
        0x13 => Some(if shift { b'R' } else { b'r' }),
        0x14 => Some(if shift { b'T' } else { b't' }),
        0x15 => Some(if shift { b'Y' } else { b'y' }),
        0x16 => Some(if shift { b'U' } else { b'u' }),
        0x17 => Some(if shift { b'I' } else { b'i' }),
        0x18 => Some(if shift { b'O' } else { b'o' }),
        0x19 => Some(if shift { b'P' } else { b'p' }),
        0x1A => Some(if shift { b'{' } else { b'[' }),
        0x1B => Some(if shift { b'}' } else { b']' }),
        0x1C => Some(b'\n'),
        0x1E => Some(if shift { b'A' } else { b'a' }),
        0x1F => Some(if shift { b'S' } else { b's' }),
        0x20 => Some(if shift { b'D' } else { b'd' }),
        0x21 => Some(if shift { b'F' } else { b'f' }),
        0x22 => Some(if shift { b'G' } else { b'g' }),
        0x23 => Some(if shift { b'H' } else { b'h' }),
        0x24 => Some(if shift { b'J' } else { b'j' }),
        0x25 => Some(if shift { b'K' } else { b'k' }),
        0x26 => Some(if shift { b'L' } else { b'l' }),
        0x27 => Some(if shift { b':' } else { b';' }),
        0x28 => Some(if shift { b'"' } else { b'\'' }),
        0x29 => Some(if shift { b'~' } else { b'`' }),
        0x2B => Some(if shift { b'|' } else { b'\\' }),
        0x2C => Some(if shift { b'Z' } else { b'z' }),
        0x2D => Some(if shift { b'X' } else { b'x' }),
        0x2E => Some(if shift { b'C' } else { b'c' }),
        0x2F => Some(if shift { b'V' } else { b'v' }),
        0x30 => Some(if shift { b'B' } else { b'b' }),
        0x31 => Some(if shift { b'N' } else { b'n' }),
        0x32 => Some(if shift { b'M' } else { b'm' }),
        0x33 => Some(if shift { b'<' } else { b',' }),
        0x34 => Some(if shift { b'>' } else { b'.' }),
        0x35 => Some(if shift { b'?' } else { b'/' }),
        0x39 => Some(b' '),
        _ => None,
    }
}

pub unsafe fn win_print_char(id: usize, c: u8, color: u8) {
    let win = &mut WINDOWS[id];
    let w = win.w;
    let h = win.h - 1;

    if c == b'\n' {
        win.cx = 0;
        win.cy += 1;
    } else if c == 0x08 {
        if win.cx > 0 {
            win.cx -= 1;
            win.screen[win.cy * w + win.cx] = (b' ' as u16) | ((color as u16) << 8);
        }
    } else {
        win.screen[win.cy * w + win.cx] = (c as u16) | ((color as u16) << 8);
        win.cx += 1;
        if win.cx >= w - 2 {
            win.cx = 0;
            win.cy += 1;
        }
    }

    if win.cy >= h - 1 {
        for y in 1..(h - 1) {
            for x in 0..w {
                win.screen[(y - 1) * w + x] = win.screen[y * w + x];
            }
        }
        for x in 0..w {
            win.screen[(h - 2) * w + x] = (b' ' as u16) | ((color as u16) << 8);
        }
        win.cy = h - 2;
    }
}

unsafe fn win_print_str(id: usize, s: &str, color: u8) {
    for b in s.bytes() {
        win_print_char(id, b, color);
    }
}

unsafe fn win_draw_char(id: usize, x: usize, y: usize, c: u8, color: u8) {
    let win = &mut WINDOWS[id];
    if x < win.w && y < win.h - 1 {
        win.screen[y * win.w + x] = (c as u16) | ((color as u16) << 8);
    }
}

pub unsafe fn win_clear(id: usize) {
    let win = &mut WINDOWS[id];
    for i in 0..(50 * 15) {
        win.screen[i] = (b' ' as u16) | (0x0F << 8);
    }
    win.cx = 0;
    win.cy = 0;
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        play_startup_sequence();

        init_mouse();
        load_euro_glyph();
        open_new_window();

        WINDOWS[0].x = 5;
        WINDOWS[0].y = 3;
        WINDOWS[1].x = 15;
        WINDOWS[1].y = 5;
        WINDOWS[2].x = 25;
        WINDOWS[2].y = 7;
    }

    let mut shift_pressed = false;
    let mut dragging: Option<(usize, i32, i32)> = None;
    let mut prev_left = false;

    loop {
        unsafe {
            if let Some((dx, dy, left, _right)) = poll_mouse() {
                MOUSE_X += dx / 2;
                MOUSE_Y -= dy / 2;

                if MOUSE_X < 0 {
                    MOUSE_X = 0;
                }
                if MOUSE_X >= VGA_WIDTH as i32 {
                    MOUSE_X = VGA_WIDTH as i32 - 1;
                }
                if MOUSE_Y < 0 {
                    MOUSE_Y = 0;
                }
                if MOUSE_Y >= VGA_HEIGHT as i32 {
                    MOUSE_Y = VGA_HEIGHT as i32 - 1;
                }

                mark_screen_dirty();

                if left {
                    if !prev_left {
                        let mut clicked = None;
                        for &i in &[FOCUSED_WIN, (FOCUSED_WIN + 1) % 3, (FOCUSED_WIN + 2) % 3] {
                            if !WINDOWS[i].active {
                                continue;
                            }
                            let win = &WINDOWS[i];
                            if MOUSE_X >= win.x as i32
                                && MOUSE_X < (win.x + win.w) as i32
                                && MOUSE_Y >= win.y as i32
                                && MOUSE_Y < (win.y + win.h) as i32
                            {
                                if MOUSE_Y == win.y as i32 {
                                    if i != 0 && MOUSE_X >= (win.x + win.w - 4) as i32 {
                                        WINDOWS[i].active = false;
                                        let mut any_active = false;
                                        for j in 0..3 {
                                            if WINDOWS[j].active {
                                                FOCUSED_WIN = j;
                                                any_active = true;
                                                break;
                                            }
                                        }
                                        if !any_active {
                                            FOCUSED_WIN = 0;
                                        }
                                    } else {
                                        dragging = Some((
                                            i,
                                            MOUSE_X - win.x as i32,
                                            MOUSE_Y - win.y as i32,
                                        ));
                                        clicked = Some(i);
                                    }
                                } else {
                                    clicked = Some(i);
                                }
                                break;
                            }
                        }
                        if let Some(i) = clicked {
                            FOCUSED_WIN = i;
                        }
                    } else {
                        if let Some((id, off_x, off_y)) = dragging {
                            let mut nx = MOUSE_X - off_x;
                            let mut ny = MOUSE_Y - off_y;
                            if nx < 0 {
                                nx = 0;
                            }
                            if ny < 0 {
                                ny = 0;
                            }
                            if nx + WINDOWS[id].w as i32 > VGA_WIDTH as i32 {
                                nx = VGA_WIDTH as i32 - WINDOWS[id].w as i32;
                            }
                            if ny + WINDOWS[id].h as i32 > VGA_HEIGHT as i32 {
                                ny = VGA_HEIGHT as i32 - WINDOWS[id].h as i32;
                            }
                            WINDOWS[id].x = nx as usize;
                            WINDOWS[id].y = ny as usize;
                        }
                    }
                } else {
                    dragging = None;
                }
                prev_left = left;
            }

            if let Some(scancode) = poll_keyboard() {
                if scancode == 0x2A || scancode == 0x36 {
                    shift_pressed = true;
                } else if scancode == 0xAA || scancode == 0xB6 {
                    shift_pressed = false;
                } else if scancode & 0x80 == 0 && FOCUSED_WIN < 3 {
                    if let Some(c) = scancode_to_char(scancode, shift_pressed) {
                        let id = FOCUSED_WIN;

                        if c == b'\n' {
                            win_print_char(id, b'\n', 0x0F);
                            let win = &mut WINDOWS[id];
                            let cmd = &win.buf[..win.buf_len];

                            if cmd == b"r.neofetch" {
                                win_print_str(id, "       _~^~^~_\n", 0x0C);
                                win_print_str(id, "   \\) /  o o  \\ (/\n", 0x0C);
                                win_print_str(id, "     '_   -   _'\n", 0x0C);
                                win_print_str(id, "     / '-----' \\\n", 0x0C);
                                win_print_str(id, "OS: ferrisOS PRIMITIVE 1\n", 0x0B);
                            } else if cmd == b"r.help" {
                                win_print_str(id, "- r.neofetch\n- r.play.ferris_dino\n- r.play.ferris_maker\n- r.play.ferrisdungeon\n- r.new\n- r.close\n- r.sd\n- r.wash\n- r.ide\n- r.claw\n- r.carapace\n- r.ad\n- r.sf\n- r.ex\n- r.desk\n-" , 0x0F);
                            } else if cmd == b"r.play.ferris_dino" {
                                play_ferrisjump(id);
                            } else if cmd == b"r.play.ferris_maker" {
                                play_ferris_maker(id);
                                win_clear(id);
                                win_print_char(id, 0xEE, 0x0A);
                                win_print_char(id, b' ', 0x0A);
                            } else if cmd == b"r.play.ferrisdungeon" {
                                play_ferrisdungeon(id);
                                win_clear(id);
                                win_print_char(id, 0xEE, 0x0A);
                                win_print_char(id, b' ', 0x0A);
                            } else if cmd == b"r.new" {
                                open_new_window();
                            } else if cmd == b"r.chest" {
                                run_chest(id);
                                win_clear(id);
                            } else if cmd == b"r.sd" {
                                outw(0x604, 0x2000);
                                asm!("hlt");
                            } else if cmd == b"r.wash" {
                                win_clear(id);
                            } else if cmd == b"r.ide" {
                                run_ide(id);
                                win_clear(id);
                            } else if cmd == b"r.carapace" {
                                run_explorer(id);
                                win_clear(id);
                            } else if cmd == b"r.close" {
                                if id != 0 {
                                    WINDOWS[id].active = false;
                                    let mut any_active = false;
                                    for j in 0..3 {
                                        if WINDOWS[j].active {
                                            FOCUSED_WIN = j;
                                            any_active = true;
                                            break;
                                        }
                                    }
                                    if !any_active {
                                        FOCUSED_WIN = 0;
                                    }
                                } else {
                                    win_print_str(id, "Cannot close the main window.\n", 0x0C);
                                }
                            } else if cmd == b"r.claw" {
                                run_claw(id);
                                win_clear(id);
                            } else if cmd.len() >= 4 && &cmd[..4] == b"r.ad" {
                                if cmd.len() == 15 && &cmd[5..14] == b"carapace." {
                                    let f_id = cmd[14];
                                    if f_id >= b'0' && f_id <= b'4' {
                                        WIN_DIR[id] = f_id - b'0';
                                    } else {
                                        win_print_str(id, "Invalid folder.\n", 0x0C);
                                    }
                                } else if cmd.len() == 13 && &cmd[5..13] == b"carapace" {
                                    WIN_DIR[id] = 0;
                                } else if cmd.len() == 7 && &cmd[5..7] == b".." {
                                    WIN_DIR[id] = 255;
                                } else {
                                    win_print_str(id, "Unknown directory.\n", 0x0C);
                                }
                            } else if cmd.len() >= 5 && &cmd[..5] == b"r.ex " {
                                if WIN_DIR[id] == 255 {
                                    win_print_str(id, "Not in a carapace folder.\n", 0x0C);
                                } else {
                                    let fname = &cmd[5..];
                                    let mut found = false;
                                    for i in 0..10 {
                                        if FILES[i].active && FILES[i].folder_id == WIN_DIR[id] {
                                            let mut n_len = 0;
                                            while n_len < 16 && FILES[i].name[n_len] != 0 {
                                                n_len += 1;
                                            }
                                            if n_len == fname.len()
                                                && &FILES[i].name[..n_len] == fname
                                            {
                                                crate::ferriscript::execute_ferriscript(
                                                    id,
                                                    &FILES[i].content,
                                                    WIN_DIR[id],
                                                );
                                                found = true;
                                                break;
                                            }
                                        }
                                    }
                                    if !found {
                                        win_print_str(id, "File not found.\n", 0x0C);
                                    }
                                }
                            } else if cmd == b"r.sf" {
                                if WIN_DIR[id] == 255 {
                                    win_print_str(id, "Not in a carapace folder.\n", 0x0C);
                                } else {
                                    let mut found_any = false;
                                    for i in 0..10 {
                                        if FILES[i].active && FILES[i].folder_id == WIN_DIR[id] {
                                            for j in 0..16 {
                                                let ch = FILES[i].name[j];
                                                if ch == 0 {
                                                    break;
                                                }
                                                win_print_char(id, ch, 0x0B);
                                            }
                                            win_print_char(id, b'\n', 0x0F);
                                            found_any = true;
                                        }
                                    }
                                    if !found_any {
                                        win_print_str(id, "Folder is empty.\n", 0x0F);
                                    }
                                }
                            } else if win.buf_len > 0 {
                                win_print_str(id, "Unknown command.\n", 0x0C);
                            };

                            WINDOWS[id].buf_len = 0;
                            if WINDOWS[id].active
                                && cmd != b"r.play/ferris_maker"
                                && cmd != b"r.play/ferris_dino"
                                && cmd != b"r.close"
                            {
                                win_print_char(id, 0xEE, 0x0A);
                                win_print_char(id, b' ', 0x0A);
                            }
                        } else if c == 0x08 {
                            if WINDOWS[id].buf_len > 0 {
                                WINDOWS[id].buf_len -= 1;
                                win_print_char(id, 0x08, 0x0F);
                            }
                        } else if WINDOWS[id].buf_len < 128 {
                            let len = WINDOWS[id].buf_len;
                            WINDOWS[id].buf[len] = c;
                            WINDOWS[id].buf_len += 1;
                            win_print_char(id, c, 0x0F);
                        }
                    }
                }
            }

            composite_screen();
        }
    }
}