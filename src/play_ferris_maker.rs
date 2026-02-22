use crate::{
    composite_screen, poll_keyboard, poll_mouse, sleep_ms, win_clear, win_draw_char, win_print_str,
    MOUSE_X, MOUSE_Y, VGA_HEIGHT, VGA_WIDTH, WINDOWS,
};

pub unsafe fn play_ferris_maker(id: usize) {
    let w = WINDOWS[id].w;
    let h = WINDOWS[id].h;
    let mut map = [b' '; 50 * 15];

    for x in 0..w {
        map[2 * w + x] = b'#';
        map[(h - 2) * w + x] = b'#';
    }
    for y in 2..h - 1 {
        map[y * w] = b'#';
        map[y * w + w - 1] = b'#';
    }

    let mut is_editing = true;
    let mut ferris_x: i32 = 5;
    let mut ferris_y: i32 = 5;

    loop {
        sleep_ms(16);

        if let Some((dx, dy, left, right)) = poll_mouse() {
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

            if is_editing {
                let wx = WINDOWS[id].x as i32;
                let wy = WINDOWS[id].y as i32;
                if MOUSE_X >= wx
                    && MOUSE_X < wx + w as i32
                    && MOUSE_Y > wy
                    && MOUSE_Y < wy + h as i32 - 1
                {
                    let lx = (MOUSE_X - wx) as usize;
                    let ly = (MOUSE_Y - wy) as usize;
                    if ly > 2 && ly < h - 2 && lx > 0 && lx < w - 1 {
                        if left {
                            map[ly * w + lx] = b'#';
                        }
                        if right {
                            map[ly * w + lx] = b' ';
                        }
                    }
                }
            }
        }

        while let Some(scancode) = poll_keyboard() {
            if scancode == 0x01 {
                return;
            } else if scancode == 0x12 {
                is_editing = true;
            } else if scancode == 0x19 {
                is_editing = false;
                ferris_x = 5;
                ferris_y = 5;
            } else if !is_editing {
                let mut nx = ferris_x;
                let mut ny = ferris_y;

                if scancode == 0x11 {
                    ny -= 1;
                }
                if scancode == 0x1F {
                    ny += 1;
                }
                if scancode == 0x1E {
                    nx -= 1;
                }
                if scancode == 0x20 {
                    nx += 1;
                }

                let mut collision = false;
                for cy in 0..2 {
                    for cx in 0..5 {
                        let check_x = nx + cx;
                        let check_y = ny + cy;
                        if check_x >= 0 && check_x < w as i32 && check_y >= 0 && check_y < h as i32
                        {
                            if map[(check_y as usize) * w + (check_x as usize)] == b'#' {
                                collision = true;
                            }
                        } else {
                            collision = true;
                        }
                    }
                }

                if !collision {
                    ferris_x = nx;
                    ferris_y = ny;
                }
            }
        }

        win_clear(id);

        let status = if is_editing {
            "[EDIT] P:Play ESC:Quit L/R:Draw "
        } else {
            "[PLAY] E:Edit ESC:Quit WASD:Move"
        };
        win_print_str(id, status, 0x0B);

        for y in 2..h - 1 {
            for x in 0..w {
                if map[y * w + x] == b'#' {
                    win_draw_char(id, x, y, 0xDB, 0x0A);
                }
            }
        }

        let s1 = b"(o_o)";
        let s2 = b"/>-<\\";
        for (i, &b) in s1.iter().enumerate() {
            win_draw_char(
                id,
                (ferris_x + i as i32) as usize,
                ferris_y as usize,
                b,
                0x0C,
            );
        }
        for (i, &b) in s2.iter().enumerate() {
            win_draw_char(
                id,
                (ferris_x + i as i32) as usize,
                (ferris_y + 1) as usize,
                b,
                0x0C,
            );
        }

        composite_screen();
    }
}
