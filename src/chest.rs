use crate::{
    composite_screen, play_sound, poll_keyboard, poll_mouse, sleep_ms, stop_sound, win_clear,
    win_draw_char, win_print_char, win_print_str, FILES, MOUSE_X, MOUSE_Y, WINDOWS, WIN_DIR,
};

pub fn run_chest(id: usize) {
    let mut notes = [255u8; 16];
    let freqs = [523, 493, 440, 392, 349, 329, 293, 261];
    let mut prev_left = false;

    let mut naming_mode = false;
    let mut name_buf = [0u8; 10];
    let mut name_len = 0;

    loop {
        unsafe {
            win_clear(id);

            if naming_mode {
                win_print_str(id, "SAVE SONG\n", 0x0E);
                win_print_str(id, "Enter File Name: ", 0x0F);
                for i in 0..name_len {
                    win_print_char(id, name_buf[i], 0x0A);
                }
                win_print_str(id, ".chest\n\n[ENTER] Confirm  [ESC] Cancel", 0x07);
            } else {
                win_print_str(id, "CHEST\n", 0x0D);
                for y in 0..8 {
                    for x in 0..16 {
                        let (ch, col) = if notes[x] == y as u8 {
                            (b'O', 0x0A)
                        } else {
                            (b'.', 0x08)
                        };
                        win_draw_char(id, x * 2 + 2, y + 3, ch, col);
                    }
                }
                win_print_str(id, "\n\n\n\n\n\n\n\n\n[P] Play  [S] Save  [ESC] Exit", 0x0F);
            }

            if let Some((dx, dy, left, _)) = poll_mouse() {
                MOUSE_X += dx / 2;
                MOUSE_Y -= dy / 2;
                if MOUSE_X < 0 {
                    MOUSE_X = 0;
                }
                if MOUSE_X > 79 {
                    MOUSE_X = 79;
                }
                if MOUSE_Y < 0 {
                    MOUSE_Y = 0;
                }
                if MOUSE_Y > 24 {
                    MOUSE_Y = 24;
                }

                if !naming_mode && left && !prev_left {
                    let win = &WINDOWS[id];
                    let (rx, ry) = (MOUSE_X - win.x as i32, MOUSE_Y - win.y as i32);
                    if ry >= 3 && ry <= 10 && rx >= 2 && rx < 34 {
                        let gx = ((rx - 2) / 2) as usize;
                        let gy = (ry - 3) as u8;
                        if gx < 16 {
                            notes[gx] = if notes[gx] == gy { 255 } else { gy };
                        }
                    }
                }
                prev_left = left;
            }

            composite_screen();

            if let Some(sc) = poll_keyboard() {
                if naming_mode {
                    match sc {
                        0x01 => {
                            naming_mode = false;
                            name_len = 0;
                        }
                        0x1C => {
                            if name_len > 0 {
                                let mut slot = 255;
                                for i in 0..10 {
                                    if !FILES[i].active {
                                        slot = i;
                                        break;
                                    }
                                }
                                if slot != 255 {
                                    FILES[slot].active = true;
                                    FILES[slot].folder_id = WIN_DIR[id];

                                    let ext = b".chest";
                                    for i in 0..16 {
                                        FILES[slot].name[i] = 0;
                                    } 
                                    for i in 0..name_len {
                                        FILES[slot].name[i] = name_buf[i];
                                    }
                                    for i in 0..6 {
                                        FILES[slot].name[name_len + i] = ext[i];
                                    }

                                    for i in 0..512 {
                                        FILES[slot].content[i] = 0;
                                    }
                                    FILES[slot].content[0] = 0xC0; 
                                    for i in 0..16 {
                                        FILES[slot].content[i + 1] = notes[i];
                                    }

                                    win_print_str(id, "\nSaved!", 0x0A);
                                    composite_screen();
                                    sleep_ms(800);
                                    naming_mode = false;
                                }
                            }
                        }
                        0x0E => {
                            if name_len > 0 {
                                name_len -= 1;
                            }
                        } 
                        _ => {
                            let chars = b"??qwertyuiop[]??asdfghjkl;''?zxcvbnm,./";
                            if sc >= 0x10 && sc <= 0x35 && name_len < 9 {
                                let c = chars[(sc - 0x10) as usize];
                                if c != b'?' {
                                    name_buf[name_len] = c;
                                    name_len += 1;
                                }
                            }
                        }
                    }
                } else {
                    match sc {
                        0x01 => return,
                        0x19 => {
                            
                            for step in 0..16 {
                                win_draw_char(id, step * 2 + 2, 2, 0x19, 0x0E);
                                composite_screen();
                                if notes[step] != 255 {
                                    play_sound(freqs[notes[step] as usize]);
                                }
                                sleep_ms(150);
                                stop_sound();
                                sleep_ms(30);
                                win_draw_char(id, step * 2 + 2, 2, b' ', 0x0F);
                                if let Some((dx, dy, _, _)) = poll_mouse() {
                                    MOUSE_X += dx / 2;
                                    MOUSE_Y -= dy / 2;
                                }
                            }
                        }
                        0x1F => {
                            naming_mode = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        sleep_ms(16);
    }
}
