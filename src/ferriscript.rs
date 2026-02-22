use crate::{
    composite_screen, poll_keyboard, sleep_ms, win_clear, win_print_char, win_print_str, FILES,
};

static mut RECURSION_DEPTH: u8 = 0;

#[derive(Copy, Clone, PartialEq)]
enum VarType {
    Str,
    U8,
    I8,
    Bool,
}

#[derive(Copy, Clone, PartialEq)]
enum TriBool {
    False,
    True,
    Maybe,
}

#[derive(Copy, Clone)]
struct Variable {
    name: [u8; 16],
    v_type: VarType,
    s_val: [u8; 32],
    u_val: u8,
    i_val: i8,
    b_val: TriBool,
    active: bool,
}

static mut VARS: [Variable; 32] = [Variable {
    name: [0; 16],
    v_type: VarType::U8,
    s_val: [0; 32],
    u_val: 0,
    i_val: 0,
    b_val: TriBool::False,
    active: false,
}; 32];

pub unsafe fn run_ide(id: usize) {
    let mut tabs = [[0u8; 512], [0u8; 512], [0u8; 512]];
    let mut lens = [0usize; 3];
    let mut current_tab = 0;

    loop {
        win_clear(id);
        win_print_str(id, "TABS: [F1-F3] | [F10]Save [F5]Run [ESC]Quit\n", 0x0E);
        for i in 0..3 {
            let color = if i == current_tab { 0x1F } else { 0x07 };
            win_print_str(id, " Tab", color);
            win_print_char(id, (i as u8 + b'1'), color);
        }
        win_print_str(id, "\n\n", 0x07);
        for i in 0..lens[current_tab] {
            win_print_char(id, tabs[current_tab][i], 0x0F);
        }
        composite_screen();
        if let Some(sc) = poll_keyboard() {
            if sc == 0x01 {
                return;
            }
            if sc == 0x3B {
                current_tab = 0;
                continue;
            }
            if sc == 0x3C {
                current_tab = 1;
                continue;
            }
            if sc == 0x3D {
                current_tab = 2;
                continue;
            }
            if sc == 0x3F {
                RECURSION_DEPTH = 0;
                execute_ferriscript(id, &tabs[current_tab][..lens[current_tab]], 0);
                win_print_str(id, "\n[Done]", 0x08);
                composite_screen();
                sleep_ms(2000);
                continue;
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
                            if n_len < 9 {
                                name[n_len] = c;
                                n_len += 1;
                                win_print_char(id, c, 0x0F);
                            }
                        }
                    }
                }
                if n_len > 0 {
                    let ext = b".ferris";
                    for i in 0..7 {
                        name[n_len + i] = ext[i];
                    }
                    save_to_vfs(name, &tabs[current_tab]);
                    win_print_str(id, "\nSaved!", 0x0A);
                    composite_screen();
                    sleep_ms(500);
                }
                continue;
            }
            if sc & 0x80 == 0 {
                if sc == 0x0E {
                    if lens[current_tab] > 0 {
                        lens[current_tab] -= 1;
                    }
                } else if sc == 0x1C {
                    if lens[current_tab] < 512 {
                        tabs[current_tab][lens[current_tab]] = b'\n';
                        lens[current_tab] += 1;
                    }
                } else if let Some(c) = crate::scancode_to_char(sc, false) {
                    if lens[current_tab] < 512 {
                        tabs[current_tab][lens[current_tab]] = c;
                        lens[current_tab] += 1;
                    }
                }
            }
        }
        sleep_ms(10);
    }
}

pub unsafe fn execute_ferriscript(id: usize, code: &[u8], folder: u8) {
    if RECURSION_DEPTH > 10 {
        return;
    }
    RECURSION_DEPTH += 1;
    let mut i = 0;
    while i < code.len() {
        while i < code.len()
            && (code[i] == b' ' || code[i] == b'\n' || code[i] == b'\r' || code[i] == b'\t')
        {
            i += 1;
        }
        if i >= code.len() {
            break;
        }

        if i + 5 <= code.len() && &code[i..i + 5] == b"print" {
            i += 5;
            while i < code.len() && code[i] == b' ' {
                i += 1;
            }
            if i + 2 <= code.len() && code[i] == b',' && code[i + 1] == b',' {
                i += 2;
                while i + 2 <= code.len() && !(code[i] == b',' && code[i + 1] == b',') {
                    win_print_char(id, code[i], 0x0F);
                    i += 1;
                }
                i += 2;
            } else {
                let start = i;
                while i < code.len() && code[i] > b' ' {
                    i += 1;
                }
                let name = &code[start..i];
                let mut found = false;
                for s in 0..32 {
                    if VARS[s].active {
                        let mut n_len = 0;
                        while n_len < 16 && VARS[s].name[n_len] != 0 {
                            n_len += 1;
                        }
                        if n_len == name.len() && &VARS[s].name[..n_len] == name {
                            match VARS[s].v_type {
                                VarType::Str => {
                                    for n in 0..32 {
                                        if VARS[s].s_val[n] == 0 {
                                            break;
                                        }
                                        win_print_char(id, VARS[s].s_val[n], 0x0F);
                                    }
                                }
                                VarType::U8 => {
                                    let mut n = VARS[s].u_val;
                                    if n == 0 {
                                        win_print_char(id, b'0', 0x0F);
                                    } else {
                                        let mut buf = [0u8; 3];
                                        let mut b_i = 0;
                                        while n > 0 {
                                            buf[b_i] = (n % 10) + b'0';
                                            n /= 10;
                                            b_i += 1;
                                        }
                                        while b_i > 0 {
                                            b_i -= 1;
                                            win_print_char(id, buf[b_i], 0x0F);
                                        }
                                    }
                                }
                                VarType::I8 => {
                                    let mut val = VARS[s].i_val;
                                    if val == 0 {
                                        win_print_char(id, b'0', 0x0F);
                                    } else {
                                        if val < 0 {
                                            win_print_char(id, b'-', 0x0F);
                                            val = -val;
                                        }
                                        let mut buf = [0u8; 3];
                                        let mut b_i = 0;
                                        let mut n = val as u8;
                                        while n > 0 {
                                            buf[b_i] = (n % 10) + b'0';
                                            n /= 10;
                                            b_i += 1;
                                        }
                                        while b_i > 0 {
                                            b_i -= 1;
                                            win_print_char(id, buf[b_i], 0x0F);
                                        }
                                    }
                                }
                                VarType::Bool => match VARS[s].b_val {
                                    TriBool::True => win_print_str(id, "true", 0x0F),
                                    TriBool::False => win_print_str(id, "false", 0x0F),
                                    TriBool::Maybe => win_print_str(id, "maybe", 0x0F),
                                },
                            }
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    for &b in name {
                        win_print_char(id, b, 0x0F);
                    }
                }
            }
            win_print_char(id, b'\n', 0x0F);
        } else if i + 4 <= code.len() && &code[i..i + 3] == b"var" {
            i += 4;
            let mut v_type = VarType::U8;
            if i + 3 <= code.len() && &code[i..i + 3] == b"str" {
                v_type = VarType::Str;
                i += 4;
            } else if i + 2 <= code.len() && &code[i..i + 2] == b"u8" {
                v_type = VarType::U8;
                i += 3;
            } else if i + 2 <= code.len() && &code[i..i + 2] == b"i8" {
                v_type = VarType::I8;
                i += 3;
            } else if i + 4 <= code.len() && &code[i..i + 4] == b"bool" {
                v_type = VarType::Bool;
                i += 5;
            }
            let name_start = i;
            while i < code.len() && code[i] != b' ' && code[i] != b'=' {
                i += 1;
            }
            let name = &code[name_start..i];
            while i < code.len() && (code[i] == b' ' || code[i] == b'=') {
                i += 1;
            }
            let mut slot = 32;
            for s in 0..32 {
                if !VARS[s].active {
                    slot = s;
                    break;
                }
                let mut n_len = 0;
                while n_len < 16 && VARS[s].name[n_len] != 0 {
                    n_len += 1;
                }
                if n_len == name.len() && &VARS[s].name[..n_len] == name {
                    slot = s;
                    break;
                }
            }
            if slot < 32 {
                VARS[slot].active = true;
                VARS[slot].v_type = v_type;
                for n in 0..16 {
                    VARS[slot].name[n] = if n < name.len() { name[n] } else { 0 };
                }
                match v_type {
                    VarType::Str => {
                        if i + 2 <= code.len() && code[i] == b',' && code[i + 1] == b',' {
                            i += 2;
                            let s_start = i;
                            while i + 2 <= code.len() && !(code[i] == b',' && code[i + 1] == b',') {
                                i += 1;
                            }
                            let s_val = &code[s_start..i];
                            for n in 0..32 {
                                VARS[slot].s_val[n] = if n < s_val.len() { s_val[n] } else { 0 };
                            }
                            i += 2;
                        }
                    }
                    VarType::U8 => {
                        let mut val = 0u16;
                        while i < code.len() && code[i] >= b'0' && code[i] <= b'9' {
                            val = val * 10 + (code[i] - b'0') as u16;
                            i += 1;
                        }
                        VARS[slot].u_val = (val % 256) as u8;
                    }
                    VarType::I8 => {
                        let mut neg = false;
                        if i < code.len() && code[i] == b'-' {
                            neg = true;
                            i += 1;
                        }
                        let mut val = 0i16;
                        while i < code.len() && code[i] >= b'0' && code[i] <= b'9' {
                            val = val * 10 + (code[i] - b'0') as i16;
                            i += 1;
                        }
                        let final_val = if neg { -val } else { val };
                        VARS[slot].i_val = (final_val.max(-128).min(127)) as i8;
                    }
                    VarType::Bool => {
                        if i + 4 <= code.len() && &code[i..i + 4] == b"true" {
                            VARS[slot].b_val = TriBool::True;
                            i += 4;
                        } else if i + 5 <= code.len() && &code[i..i + 5] == b"false" {
                            VARS[slot].b_val = TriBool::False;
                            i += 5;
                        } else if i + 5 <= code.len() && &code[i..i + 5] == b"maybe" {
                            VARS[slot].b_val = TriBool::Maybe;
                            i += 5;
                        }
                    }
                }
            }
        } else if i + 4 <= code.len() && (&code[i..i + 3] == b"add" || &code[i..i + 3] == b"sub") {
            let is_add = &code[i..i + 3] == b"add";
            i += 4;
            let start = i;
            while i < code.len() && code[i] != b' ' {
                i += 1;
            }
            let name = &code[start..i];
            while i < code.len() && code[i] == b' ' {
                i += 1;
            }
            let mut val = 0i16;
            let mut neg = false;
            if i < code.len() && code[i] == b'-' {
                neg = true;
                i += 1;
            }
            while i < code.len() && code[i] >= b'0' && code[i] <= b'9' {
                val = val * 10 + (code[i] - b'0') as i16;
                i += 1;
            }
            if neg {
                val = -val;
            }

            for s in 0..32 {
                if VARS[s].active {
                    let mut n_len = 0;
                    while n_len < 16 && VARS[s].name[n_len] != 0 {
                        n_len += 1;
                    }
                    if n_len == name.len() && &VARS[s].name[..n_len] == name {
                        if VARS[s].v_type == VarType::U8 {
                            if is_add {
                                VARS[s].u_val = VARS[s].u_val.wrapping_add(val as u8);
                            } else {
                                VARS[s].u_val = VARS[s].u_val.wrapping_sub(val as u8);
                            }
                        } else if VARS[s].v_type == VarType::I8 {
                            if is_add {
                                VARS[s].i_val = VARS[s].i_val.wrapping_add(val as i8);
                            } else {
                                VARS[s].i_val = VARS[s].i_val.wrapping_sub(val as i8);
                            }
                        }
                        break;
                    }
                }
            }
        } else if i + 8 <= code.len() && &code[i..i + 7] == b"call fn" {
            i += 8;
            let start = i;
            while i < code.len() && code[i] > b' ' {
                i += 1;
            }
            find_and_run_fn(id, &code[start..i], folder);
        } else if i + 10 <= code.len() && &code[i..i + 10] == b"loop.loop/" {
            i += 10;
            let start = i;

            
            while i < code.len() && code[i] != b'\n' && code[i] != b'\r' && code[i] != 0 {
                i += 1;
            }

            
            let mut loop_buf = [0u8; 512];
            let mut buf_i = 0;
            for j in start..i {
                if buf_i < 512 {
                    loop_buf[buf_i] = code[j];
                    buf_i += 1;
                }
            }

            
            loop {
                
                composite_screen();
                sleep_ms(16); 

                
                if let Some(sc) = poll_keyboard() {
                    if sc == 0x01 {
                        win_print_str(id, "\n[Loop Terminated]\n", 0x0C);
                        break;
                    }
                }

                
                execute_ferriscript(id, &loop_buf, folder);
            }
        } else {
            i += 1;
        }
    }
    RECURSION_DEPTH -= 1;
}

unsafe fn find_and_run_fn(id: usize, name: &[u8], folder: u8) {
    for f in 0..10 {
        if FILES[f].active && FILES[f].folder_id == folder {
            let content = &FILES[f].content;
            let mut j = 0;
            while j + 4 + name.len() <= 512 {
                if &content[j..j + 3] == b"fn/"
                    && &content[j + 3..j + 3 + name.len()] == name
                    && content[j + 3 + name.len()] == b'/'
                {
                    let start_body = j + 3 + name.len() + 1;
                    let mut k = start_body;
                    while k + 3 <= 512 && &content[k..k + 3] != b"fn/" {
                        k += 1;
                    }
                    execute_ferriscript(id, &content[start_body..k], folder);
                    return;
                }
                j += 1;
            }
        }
    }
}

pub unsafe fn save_to_vfs(name: [u8; 16], buf: &[u8; 512]) {
    for i in 0..10 {
        if !FILES[i].active || FILES[i].name == name {
            FILES[i].active = true;
            FILES[i].name = name;
            FILES[i].content = *buf;
            break;
        }
    }
}
