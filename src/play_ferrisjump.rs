use crate::{
    composite_screen, mark_screen_dirty, play_sound, poll_keyboard, poll_mouse, sleep_ms,
    stop_sound, win_clear, win_draw_char, win_print_char, win_print_str, MOUSE_X, MOUSE_Y,
    VGA_HEIGHT, VGA_WIDTH, WINDOWS,
};

pub unsafe fn play_ferrisjump(id: usize) {
    let w = WINDOWS[id].w;
    let ground_y = WINDOWS[id].h - 3;
    let mut ferris_y: i32 = ground_y as i32;
    let mut velocity: i32 = 0;
    let mut is_jumping = false;
    let mut obs_x: i32 = (w - 3) as i32;
    let mut score: u32 = 0;
    let mut tick: u32 = 0;
    let mut sound_timer: u32 = 0;
    let mut cloud_x: i32 = (w - 10) as i32;
    let ferris_x: i32 = 5;

    loop {
        sleep_ms(16);
        tick += 1;

        if sound_timer > 0 {
            sound_timer -= 1;
            if sound_timer == 0 {
                stop_sound();
            }
        }

        if let Some((dx, dy, _, _)) = poll_mouse() {
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
        }

        while let Some(scancode) = poll_keyboard() {
            if scancode == 0x39 && !is_jumping {
                velocity = -3;
                is_jumping = true;
                tick = 0;
            } else if scancode == 0x01 {
                stop_sound();
                return;
            }
        }

        if is_jumping {
            if tick % 2 == 0 {
                ferris_y += velocity;
                velocity += 1;
            }
        }

        if ferris_y >= ground_y as i32 {
            ferris_y = ground_y as i32;
            is_jumping = false;
            velocity = 0;
        }

        if tick % 2 == 0 {
            obs_x -= 1;
            if obs_x < 0 {
                obs_x = (w - 3) as i32;
                score += 1;
                play_sound(1046);
                sound_timer = 4;
            }
        }

        if tick % 6 == 0 {
            cloud_x -= 1;
            if cloud_x < 0 {
                cloud_x = (w - 5) as i32;
            }
        }

        if obs_x >= ferris_x && obs_x <= ferris_x + 2 && ferris_y >= ground_y as i32 {
            stop_sound();
            play_sound(150);
            sleep_ms(300);
            stop_sound();
            break;
        }

        win_clear(id);
        win_print_str(id, "SCORE: ", 0x0F);

        let mut n = score;
        if n == 0 {
            win_print_char(id, b'0', 0x0F);
        } else {
            let mut buf = [0u8; 10];
            let mut i = 0;
            while n > 0 {
                buf[i] = (n % 10) as u8 + b'0';
                n /= 10;
                i += 1;
            }
            while i > 0 {
                i -= 1;
                win_print_char(id, buf[i], 0x0F);
            }
        }

        win_draw_char(id, cloud_x as usize, 3, 0xDF, 0x0F);
        win_draw_char(id, (cloud_x + 1) as usize, 3, 0xDF, 0x0F);
        win_draw_char(id, (cloud_x + 2) as usize, 3, 0xDF, 0x0F);

        for i in 0..w {
            win_draw_char(id, i, ground_y + 1, 0xCD, 0x0E);
        }

        win_draw_char(id, obs_x as usize, ground_y, 0xDB, 0x0A);

        let s1 = b"o_o";
        let s2 = b">_<";
        for (i, &b) in s1.iter().enumerate() {
            win_draw_char(
                id,
                (ferris_x + i as i32) as usize,
                (ferris_y - 1) as usize,
                b,
                0x0C,
            );
        }
        for (i, &b) in s2.iter().enumerate() {
            win_draw_char(
                id,
                (ferris_x + i as i32) as usize,
                ferris_y as usize,
                b,
                0x0C,
            );
        }

        composite_screen();
    }

    win_clear(id);
    win_print_str(id, "GAME OVER! SCORE: ", 0x0C);
    let mut n = score;
    if n == 0 {
        win_print_char(id, b'0', 0x0C);
    } else {
        let mut buf = [0u8; 10];
        let mut i = 0;
        while n > 0 {
            buf[i] = (n % 10) as u8 + b'0';
            n /= 10;
            i += 1;
        }
        while i > 0 {
            i -= 1;
            win_print_char(id, buf[i], 0x0C);
        }
    }
    win_print_char(id, b'\n', 0x0A);
    win_print_char(id, 0xEE, 0x0A);
    win_print_char(id, b' ', 0x0A);
    composite_screen();
}
