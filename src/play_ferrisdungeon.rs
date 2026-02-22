use crate::{composite_screen, poll_keyboard, sleep_ms, win_clear, win_draw_char, WINDOWS};

#[derive(Copy, Clone)]
struct Enemy {
    x: usize,
    y: usize,
    hp: i32,
    active: bool,
    kind: u8,
}

pub unsafe fn play_ferrisdungeon(id: usize) {
    let w = WINDOWS[id].w;
    let h = WINDOWS[id].h;

    win_clear(id);

    let title = b"FERRIS DUNGEON";
    for (i, &b) in title.iter().enumerate() {
        win_draw_char(id, (w / 2) - 7 + i, (h / 2) - 3, b, 0x0E);
    }

    let crab1 = b"(o_o)";
    let crab2 = b"/>-<\\";
    for (i, &b) in crab1.iter().enumerate() {
        win_draw_char(id, (w / 2) - 2 + i, (h / 2) - 1, b, 0x0C);
    }
    for (i, &b) in crab2.iter().enumerate() {
        win_draw_char(id, (w / 2) - 2 + i, (h / 2), b, 0x0C);
    }

    let p_start = b"Press [ENTER] to delve";
    for (i, &b) in p_start.iter().enumerate() {
        win_draw_char(id, (w / 2) - 11 + i, (h / 2) + 2, b, 0x0F);
    }

    let p_esc = b"Press [ESC] to flee";
    for (i, &b) in p_esc.iter().enumerate() {
        win_draw_char(id, (w / 2) - 9 + i, (h / 2) + 4, b, 0x08);
    }

    composite_screen();

    loop {
        if let Some(sc) = poll_keyboard() {
            if sc == 0x01 {
                return;
            }
            if sc == 0x1C {
                break;
            }
        }
        sleep_ms(10);
    }

    let mut map = [b' '; 2000];
    let mut enemies = [Enemy {
        x: 0,
        y: 0,
        hp: 0,
        active: false,
        kind: 0,
    }; 15];

    let mut hp = 25;
    let max_hp = 25;
    let mut depth = 1;
    let mut px = 2;
    let mut py = 2;
    let mut seed: u32 = 918273;

    let mut next_rand = || -> u32 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        seed
    };

    let mut generate_level = true;

    loop {
        if generate_level {
            generate_level = false;
            for i in 0..(w * h) {
                map[i] = b' ';
            }
            for x in 0..w {
                map[x] = b'#';
                map[(h - 2) * w + x] = b'#';
            }
            for y in 0..h - 1 {
                map[y * w] = b'#';
                map[y * w + w - 1] = b'#';
            }

            for _ in 0..(w * h / 60) {
                let rx = ((next_rand() as usize % ((w - 10) / 8)) * 8) + 4;
                let ry = ((next_rand() as usize % ((h - 6) / 4)) * 4) + 2;

                if rx + 1 < w && ry + 1 < h {
                    map[ry * w + rx] = b'#';
                    map[ry * w + rx + 1] = b'#';
                    map[(ry + 1) * w + rx] = b'#';
                    map[(ry + 1) * w + rx + 1] = b'#';
                }
            }

            px = 2;
            py = 2;
            for cy in 0..2 {
                for cx in 0..5 {
                    map[(py + cy) * w + (px + cx)] = b' ';
                }
            }

            let mut ex = 0;
            let mut ey = 0;
            loop {
                ex = (next_rand() as usize % (w - 10)) + 3;
                ey = (next_rand() as usize % (h - 6)) + 2;
                if ex > px + 10 || ey > py + 5 {
                    break;
                }
            }
            map[ey * w + ex] = b'>';
            map[ey * w + ex + 1] = b'>';

            for i in 0..15 {
                enemies[i].active = false;
            }
            let num_enemies = (depth + 2).min(15);
            for i in 0..num_enemies {
                let mut enx = 0;
                let mut eny = 0;
                loop {
                    enx = (next_rand() as usize % (w - 10)) + 3;
                    eny = (next_rand() as usize % (h - 6)) + 2;
                    let overlap = enx < px + 5 && enx + 3 > px && eny < py + 2 && eny + 2 > py;
                    if !overlap && map[eny * w + enx] == b' ' {
                        break;
                    }
                }
                enemies[i] = Enemy {
                    x: enx,
                    y: eny,
                    hp: depth as i32 * 2,
                    active: true,
                    kind: 0,
                };
            }
        }

        win_clear(id);

        for y in 0..h - 1 {
            for x in 0..w {
                let t = map[y * w + x];
                if t == b'#' {
                    win_draw_char(id, x, y, 0xDB, 0x08);
                } else if t == b'>' {
                    win_draw_char(id, x, y, 0xF0, 0x0E);
                } else {
                    win_draw_char(id, x, y, 0xFA, 0x07);
                }
            }
        }

        for i in 0..15 {
            if enemies[i].active {
                let ex = enemies[i].x;
                let ey = enemies[i].y;
                let c = 0x0A;
                win_draw_char(id, ex, ey, b'\\', c);
                win_draw_char(id, ex + 1, ey, b'm', c);
                win_draw_char(id, ex + 2, ey, b'/', c);
                win_draw_char(id, ex, ey + 1, b'(', c);
                win_draw_char(id, ex + 1, ey + 1, b'O', 0x0C);
                win_draw_char(id, ex + 2, ey + 1, b')', c);
            }
        }

        let s1 = b"(o_o)";
        let s2 = b"/>-<\\";
        for (i, &b) in s1.iter().enumerate() {
            win_draw_char(id, px + i, py, b, 0x0C);
        }
        for (i, &b) in s2.iter().enumerate() {
            win_draw_char(id, px + i, py + 1, b, 0x0C);
        }

        let hp_s = b"HP:";
        let mut ux = 0;
        for &b in hp_s {
            win_draw_char(id, ux, h - 1, b, 0x0C);
            ux += 1;
        }

        let mut n = hp;
        if n <= 0 {
            win_draw_char(id, ux, h - 1, b'0', 0x0F);
            ux += 1;
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
                win_draw_char(id, ux, h - 1, buf[i], 0x0F);
                ux += 1;
            }
        }

        ux += 2;
        let d_s = b"FLR:";
        for &b in d_s {
            win_draw_char(id, ux, h - 1, b, 0x0B);
            ux += 1;
        }

        let mut n = depth;
        if n <= 0 {
            win_draw_char(id, ux, h - 1, b'0', 0x0F);
            ux += 1;
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
                win_draw_char(id, ux, h - 1, buf[i], 0x0F);
                ux += 1;
            }
        }

        if hp <= 0 {
            let dead_s = b" [YOU DIED - ESC]";
            for &b in dead_s {
                win_draw_char(id, ux, h - 1, b, 0x04);
                ux += 1;
            }
            composite_screen();
            loop {
                if let Some(sc) = poll_keyboard() {
                    if sc == 0x01 {
                        return;
                    }
                }
                sleep_ms(10);
            }
        }

        composite_screen();

        let mut moved = false;
        let mut nx = px;
        let mut ny = py;

        loop {
            if let Some(sc) = poll_keyboard() {
                if sc == 0x01 {
                    return;
                }
                if sc == 0x11 {
                    ny -= 1;
                    moved = true;
                    break;
                }
                if sc == 0x1F {
                    ny += 1;
                    moved = true;
                    break;
                }
                if sc == 0x1E {
                    nx -= 1;
                    moved = true;
                    break;
                }
                if sc == 0x20 {
                    nx += 1;
                    moved = true;
                    break;
                }
            }
            sleep_ms(10);
        }

        if moved {
            let mut hit = false;
            for i in 0..15 {
                if enemies[i].active {
                    let ex = enemies[i].x;
                    let ey = enemies[i].y;
                    if nx < ex + 3 && nx + 5 > ex && ny < ey + 2 && ny + 2 > ey {
                        enemies[i].hp -= 5;
                        if enemies[i].hp <= 0 {
                            enemies[i].active = false;
                        }
                        hit = true;
                        break;
                    }
                }
            }

            if hit {
                let a1 = b"(>_<)";
                let a2 = b"\\>-</";
                for (i, &b) in a1.iter().enumerate() {
                    win_draw_char(id, px + i, py, b, 0x0C);
                }
                for (i, &b) in a2.iter().enumerate() {
                    win_draw_char(id, px + i, py + 1, b, 0x0C);
                }
                composite_screen();
                sleep_ms(150);
            } else {
                let mut blocked = false;
                for cy in 0..2 {
                    for cx in 0..5 {
                        if map[(ny + cy) * w + (nx + cx)] == b'#' {
                            blocked = true;
                        }
                    }
                }
                if !blocked {
                    px = nx;
                    py = ny;
                }
            }

            let mut on_stairs = false;
            for cy in 0..2 {
                for cx in 0..5 {
                    if map[(py + cy) * w + (px + cx)] == b'>' {
                        on_stairs = true;
                    }
                }
            }

            if on_stairs {
                depth += 1;
                if hp < max_hp {
                    hp += 5;
                    if hp > max_hp {
                        hp = max_hp;
                    }
                }
                generate_level = true;
                continue;
            }

            for i in 0..15 {
                if enemies[i].active {
                    let ex = enemies[i].x;
                    let ey = enemies[i].y;
                    let overlap = px < ex + 3 && px + 5 > ex && py < ey + 2 && py + 2 > ey;

                    if overlap {
                        hp -= 1;
                    } else if next_rand() % 2 == 0 {
                        let mut nex = ex;
                        let mut ney = ey;
                        let pcx = px + 2;
                        let pcy = py;
                        let ecx = ex + 1;
                        let ecy = ey;

                        if pcx > ecx {
                            nex += 1;
                        } else if pcx < ecx {
                            nex -= 1;
                        }
                        if pcy > ecy {
                            ney += 1;
                        } else if pcy < ecy {
                            ney -= 1;
                        }

                        let mut blocked = false;
                        for cy in 0..2 {
                            for cx in 0..3 {
                                if map[(ney + cy) * w + (nex + cx)] == b'#' {
                                    blocked = true;
                                }
                            }
                        }
                        for j in 0..15 {
                            if i != j
                                && enemies[j].active
                                && nex < enemies[j].x + 3
                                && nex + 3 > enemies[j].x
                                && ney < enemies[j].y + 2
                                && ney + 2 > enemies[j].y
                            {
                                blocked = true;
                            }
                        }
                        let noverlap = px < nex + 3 && px + 5 > nex && py < ney + 2 && py + 2 > ney;
                        if !blocked && !noverlap {
                            enemies[i].x = nex;
                            enemies[i].y = ney;
                        } else if noverlap {
                            hp -= 1;
                        }
                    }
                }
            }
        }
    }
}
