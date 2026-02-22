use crate::{win_clear, win_print_char, win_print_str, FOCUSED_WIN, WINDOWS};

pub unsafe fn open_new_window() {
    for i in 0..3 {
        if !WINDOWS[i].active {
            WINDOWS[i].active = true;
            WINDOWS[i].buf_len = 0;
            win_clear(i);
            win_print_str(i, "Welcome to ferrisOS.\n", 0x0A);
            win_print_str(i, "Type r.help for commands.\n", 0x0A);
            win_print_char(i, 0xEE, 0x0A);
            win_print_char(i, b' ', 0x0A);
            FOCUSED_WIN = i;
            return;
        }
    }
    win_print_str(FOCUSED_WIN, "Max windows reached.\n", 0x0C);
}
