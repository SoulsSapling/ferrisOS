use crate::{play_sound, sleep_ms, stop_sound, BACKBUFFER, VGA_ADDRESS, VGA_HEIGHT, VGA_WIDTH};

pub unsafe fn play_startup_sequence() {
    for i in 0..2000 {
        BACKBUFFER[i] = (b' ' as u16) | (0x00 << 8);
    }

    let logo = [
        "        _~^~^~_         ",
        "    \\) /  o o  \\ (/     ",
        "      '_   -   _'       ",
        "      / '-----' \\       ",
        "                        ",
        "     F E R R I S      ",
        "         O S          ",
    ];

    let start_y = (VGA_HEIGHT / 2) - (logo.len() / 2);

    for (line_idx, text) in logo.iter().enumerate() {
        let start_x = (VGA_WIDTH / 2) - (text.len() / 2);
        for (char_idx, byte) in text.bytes().enumerate() {
            let idx = (start_y + line_idx) * VGA_WIDTH + (start_x + char_idx);
            let color = if line_idx < 4 {
                0x0C
            } else if line_idx < 7 {
                0x0F
            } else {
                0x03
            };
            BACKBUFFER[idx] = (byte as u16) | ((color as u16) << 8);
        }
    }

    for i in 0..2000 {
        *(VGA_ADDRESS as *mut u16).add(i) = BACKBUFFER[i];
    }

    
    let notes = [
        (392, 120), 
        (523, 150), 
        (523, 150), 
        (587, 120), 
        (659, 120), 
        (523, 350), 
    ];

    for (freq, duration) in notes {
        if freq == 0 {
            
            sleep_ms(duration);
        } else {
            play_sound(freq);
            sleep_ms(duration);
            stop_sound();
        }
        
        sleep_ms(20);
    }

    sleep_ms(3000);
}
