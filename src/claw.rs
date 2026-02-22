use crate::{composite_screen, poll_keyboard, sleep_ms, win_clear, win_print_char, win_print_str};

pub unsafe fn run_claw(id: usize) {
    let mut buf = [0u8; 512];
    let mut len = 0usize;

    loop {
        win_clear(id);
        win_print_str(id, "CLAW EDITOR | [F10]Save [ESC]Quit\n\n", 0x0E);

        for i in 0..len {
            win_print_char(id, buf[i], 0x0F);
        }

        composite_screen();

        if let Some(sc) = poll_keyboard() {
            if sc == 0x01 {
                return;
            }
            if sc == 0x44 {
                win_print_str(id, "\nSave as: ", 0x0B);
                let mut name = [0u8; 16];
                let mut n_len = 0;
                let mut timeout = 5000;
                loop {
                    composite_screen();
                    sleep_ms(10);
                    timeout -= 1;
                    if timeout == 0 {
                        break;
                    }
                    if let Some(n_sc) = poll_keyboard() {
                        timeout = 5000;
                        if n_sc == 0x1C {
                            break;
                        }
                        if n_sc == 0x01 {
                            n_len = 0;
                            break;
                        }
                        if let Some(c) = crate::scancode_to_char(n_sc, false) {
                            if n_len < 11 {
                                name[n_len] = c;
                                n_len += 1;
                                win_print_char(id, c, 0x0F);
                            }
                        }
                    }
                }
                if n_len > 0 {
                    let ext = b".claw";
                    for i in 0..5 {
                        name[n_len + i] = ext[i];
                    }
                    crate::save_to_vfs(name, &buf);
                    win_print_str(id, "\nSaved!", 0x0A);
                    composite_screen();
                    sleep_ms(500);
                }
                continue;
            }
            if sc & 0x80 == 0 {
                if sc == 0x0E {
                    if len > 0 {
                        len -= 1;
                    }
                } else if sc == 0x1C {
                    if len < 512 {
                        buf[len] = b'\n';
                        len += 1;
                    }
                } else if let Some(c) = crate::scancode_to_char(sc, false) {
                    if len < 512 {
                        buf[len] = c;
                        len += 1;
                    }
                }
            }
        }
        sleep_ms(10);
    }
}
