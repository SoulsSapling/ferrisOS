use crate::{
    BACKBUFFER, FOCUSED_WIN, MOUSE_X, MOUSE_Y, VGA_ADDRESS, VGA_HEIGHT, VGA_WIDTH, WINDOWS,
};

static mut LAST_BACKBUFFER: [u16; 2000] = [0; 2000];
static mut NEEDS_REDRAW: bool = true;

pub unsafe fn composite_screen() {
    let mut changed = NEEDS_REDRAW;
    NEEDS_REDRAW = false;

    for i in 0..2000 {
        BACKBUFFER[i] = (0xDB as u16) | (0x0C << 8);
    }

    let draw_order = [(FOCUSED_WIN + 1) % 3, (FOCUSED_WIN + 2) % 3, FOCUSED_WIN];

    for &i in &draw_order {
        if !WINDOWS[i].active {
            continue;
        }
        let win = &WINDOWS[i];
        let title_color = if i == FOCUSED_WIN { 0x1F } else { 0x70 };

        for wy in 0..win.h {
            for wx in 0..win.w {
                let sx = win.x + wx;
                let sy = win.y + wy;
                if sx < VGA_WIDTH && sy < VGA_HEIGHT {
                    let idx = sy * VGA_WIDTH + sx;
                    if wy == 0 {
                        BACKBUFFER[idx] = (b' ' as u16) | ((title_color as u16) << 8);
                    } else {
                        BACKBUFFER[idx] = win.screen[(wy - 1) * win.w + wx];
                    }
                }
            }
        }

        let t_text = b" Terminal ";
        for (tx, &b) in t_text.iter().enumerate() {
            if win.x + 2 + tx < VGA_WIDTH {
                BACKBUFFER[win.y * VGA_WIDTH + win.x + 2 + tx] =
                    (b as u16) | ((title_color as u16) << 8);
            }
        }

        if i != 0 && win.x + win.w - 3 < VGA_WIDTH {
            BACKBUFFER[win.y * VGA_WIDTH + win.x + win.w - 3] = (b'X' as u16) | (0x4F << 8);
        }

        if i == FOCUSED_WIN {
            let cx = win.x + win.cx;
            let cy = win.y + 1 + win.cy;
            if cx < VGA_WIDTH && cy < VGA_HEIGHT {
                let idx = cy * VGA_WIDTH + cx;
                BACKBUFFER[idx] = (BACKBUFFER[idx] & 0x00FF) | (0xF0 << 8);
            }
        }
    }

    if MOUSE_X >= 0 && MOUSE_X < VGA_WIDTH as i32 && MOUSE_Y >= 0 && MOUSE_Y < VGA_HEIGHT as i32 {
        let idx = (MOUSE_Y as usize) * VGA_WIDTH + (MOUSE_X as usize);
        BACKBUFFER[idx] = (b'+' as u16) | (0x0C << 8);
    }

    
    for i in 0..2000 {
        if BACKBUFFER[i] != LAST_BACKBUFFER[i] {
            changed = true;
            break;
        }
    }

    if changed {
        for i in 0..2000 {
            *(VGA_ADDRESS as *mut u16).add(i) = BACKBUFFER[i];
            LAST_BACKBUFFER[i] = BACKBUFFER[i];
        }
    }
}

pub unsafe fn mark_screen_dirty() {
    NEEDS_REDRAW = true;
}
