use crate::{
    composite_screen, poll_keyboard, poll_mouse, sleep_ms, win_clear, win_print_char,
    win_print_str, FILES, MOUSE_X, MOUSE_Y,
};

pub unsafe fn run_explorer(id: usize) {
    let mut sel = 0;
    loop {
        win_clear(id);
        win_print_str(id, "CARAPACE: [M] Folder [Enter] Run\n", 0x0B);

        let mut count = 0;
        let mut indices = [0usize; 10];
        for i in 0..10 {
            if FILES[i].active {
                indices[count] = i;
                let color = if count == sel { 0x1F } else { 0x07 };
                win_print_str(id, " [Fld:", color);
                win_print_char(id, FILES[i].folder_id + b'0', color);
                win_print_str(id, "] ", color);

                for &b in FILES[i].name.iter() {
                    if b == 0 {
                        break;
                    }
                    win_print_char(id, b, color);
                }
                win_print_str(id, "\n", 0x07);
                count += 1;
            }
        }

        if count == 0 {
            win_print_str(id, " (No files)\n", 0x04);
        }

        
        if let Some((dx, dy, _, _)) = poll_mouse() {
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
        }

        composite_screen();

        if let Some(sc) = poll_keyboard() {
            match sc {
                0x01 => return,
                0x48 => {
                    if sel > 0 {
                        sel -= 1
                    }
                }
                0x50 => {
                    if sel < count - 1 {
                        sel += 1
                    }
                }
                0x32 => {
                    if count > 0 {
                        let idx = indices[sel];
                        FILES[idx].folder_id = (FILES[idx].folder_id + 1) % 5;
                    }
                }
                0x1C => {
                    if count > 0 {
                        let idx = indices[sel];
                        let mut is_claw = false;
                        for i in 0..11 {
                            if FILES[idx].name[i] == b'.'
                                && FILES[idx].name[i + 1] == b'c'
                                && FILES[idx].name[i + 2] == b'l'
                                && FILES[idx].name[i + 3] == b'a'
                                && FILES[idx].name[i + 4] == b'w'
                            {
                                is_claw = true;
                                break;
                            }
                        }

                        win_clear(id);
                        if is_claw {
                            win_print_str(id, "Viewing Text File\n\n", 0x0B);
                            for &b in FILES[idx].content.iter() {
                                if b == 0 {
                                    break;
                                }
                                win_print_char(id, b, 0x0F);
                            }
                        } else {
                            crate::ferriscript::execute_ferriscript(
                                id,
                                &FILES[idx].content,
                                FILES[idx].folder_id,
                            );
                        }

                        win_print_str(id, "\n\n[Press ESC to return]", 0x08);

                        
                        loop {
                            if let Some((dx, dy, _, _)) = poll_mouse() {
                                MOUSE_X += dx / 2;
                                MOUSE_Y -= dy / 2;
                            }
                            composite_screen();
                            if let Some(exit_sc) = poll_keyboard() {
                                if exit_sc == 0x01 {
                                    break;
                                }
                            }
                            sleep_ms(16);
                        }
                    }
                }
                _ => {}
            }
        }
        sleep_ms(16);
    }
}
