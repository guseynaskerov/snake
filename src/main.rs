use macroquad::prelude::*;

const CELL: f32 = 20.0;
const COLS: i32 = 30;
const ROWS: i32 = 20;
const HEADER: f32 = 40.0; // высота верхней панели
const STEP_TIME: f64 = 0.13;

fn window_conf() -> Conf {
    Conf {
        window_title: "Snake".to_owned(),
        window_width: (COLS as f32 * CELL) as i32,
        window_height: (ROWS as f32 * CELL + HEADER) as i32,
        window_resizable: false,
        ..Default::default()
    }
}

fn new_food(snake: &Vec<(i32, i32)>) -> (i32, i32) {
    loop {
        let f = (rand::gen_range(0, COLS), rand::gen_range(0, ROWS));
        if !snake.contains(&f) {
            return f;
        }
    }
}

// Центр клетки в пикселях
fn center(c: (i32, i32)) -> (f32, f32) {
    (
        c.0 as f32 * CELL + CELL / 2.0,
        HEADER + c.1 as f32 * CELL + CELL / 2.0,
    )
}

// Плавный переход между двумя цветами (t от 0.0 до 1.0)
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        1.0,
    )
}

// Рисуем текст по центру окна по горизонтали
fn draw_centered(text: &str, y: f32, size: u16, color: Color) {
    let d = measure_text(text, None, size, 1.0);
    draw_text(text, (screen_width() - d.width) / 2.0, y, size as f32, color);
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    let mut snake: Vec<(i32, i32)> = vec![(5, 10), (4, 10), (3, 10)];
    let mut dir: (i32, i32) = (1, 0);
    let mut food = new_food(&snake);
    let mut score = 0;
    let mut best = 0;
    let mut game_over = false;
    let mut last_step = get_time();
    let mut prev_snake = snake.clone(); // змейка до последнего шага

    // Цвета (r, g, b, прозрачность)
    let grass1 = Color::from_rgba(120, 200, 90, 255);
    let grass2 = Color::from_rgba(105, 185, 80, 255);
    let head_color = Color::from_rgba(40, 140, 255, 255);
    let tail_color = Color::from_rgba(30, 200, 140, 255);

    loop {
        // Управление
        if is_key_pressed(KeyCode::Right) && dir != (-1, 0) {
            dir = (1, 0);
        }
        if is_key_pressed(KeyCode::Left) && dir != (1, 0) {
            dir = (-1, 0);
        }
        if is_key_pressed(KeyCode::Up) && dir != (0, 1) {
            dir = (0, -1);
        }
        if is_key_pressed(KeyCode::Down) && dir != (0, -1) {
            dir = (0, 1);
        }

        // Перезапуск
        if game_over && is_key_pressed(KeyCode::Enter) {
            snake = vec![(5, 10), (4, 10), (3, 10)];
            prev_snake = snake.clone();
            dir = (1, 0);
            food = new_food(&snake);
            score = 0;
            game_over = false;
            last_step = get_time();
        }

        // Шаг змейки
        if !game_over && get_time() - last_step > STEP_TIME {
            last_step = get_time();

            let head = snake[0];
            let new_head = (head.0 + dir.0, head.1 + dir.1);

            let hit_wall =
                new_head.0 < 0 || new_head.0 >= COLS || new_head.1 < 0 || new_head.1 >= ROWS;

            if hit_wall || snake.contains(&new_head) {
                game_over = true;
                if score > best {
                    best = score;
                }
            } else {
                prev_snake = snake.clone(); // запоминаем, где была змейка
                snake.insert(0, new_head);
                if new_head == food {
                    score += 1;
                    food = new_food(&snake);
                } else {
                    snake.pop();
                }
            }
        }

        // ---------- Рисование ----------
        clear_background(Color::from_rgba(30, 40, 30, 255));

        // Поле в шахматном порядке
        for x in 0..COLS {
            for y in 0..ROWS {
                let c = if (x + y) % 2 == 0 { grass1 } else { grass2 };
                draw_rectangle(x as f32 * CELL, HEADER + y as f32 * CELL, CELL, CELL, c);
            }
        }

        // Яблоко: тень, тело, блик, веточка, листик (слегка "дышит")
        let (fx, fy) = center(food);
        let pulse = (get_time() * 4.0).sin() as f32 * 0.6;
        draw_circle(fx + 1.5, fy + 2.5, 8.0, Color::new(0.0, 0.0, 0.0, 0.25));
        draw_circle(fx, fy, 8.0 + pulse, Color::from_rgba(230, 40, 50, 255));
        draw_circle(fx - 3.0, fy - 3.0, 2.5, Color::new(1.0, 1.0, 1.0, 0.6));
        draw_line(fx, fy - 7.0, fx + 1.0, fy - 11.0, 2.0, Color::from_rgba(110, 70, 30, 255));
        draw_circle(fx + 4.0, fy - 9.0, 3.0, Color::from_rgba(40, 150, 40, 255));

                // Насколько шаг "прошёл": 0.0 - только начался, 1.0 - закончился
        let t = if game_over {
            1.0
        } else {
            ((get_time() - last_step) / STEP_TIME).min(1.0) as f32
        };

        // Плавные позиции сегментов: между старой и новой клеткой
        let mut pts: Vec<(f32, f32)> = Vec::new();
        for i in 0..snake.len() {
            // у нового сегмента (после еды) старой позиции нет, берём последнюю
            let old = prev_snake[i.min(prev_snake.len() - 1)];
            let (ox, oy) = center(old);
            let (ax, ay) = center(snake[i]);
            pts.push((ox + (ax - ox) * t, oy + (ay - oy) * t));
        }

        // Тень
        for &(x, y) in pts.iter() {
            draw_circle(x + 1.5, y + 2.5, 9.0, Color::new(0.0, 0.0, 0.0, 0.2));
        }

        // Тело: от хвоста к голове
        let len = snake.len() as f32;
        for i in (0..pts.len()).rev() {
            let k = 1.0 - i as f32 / len;
            let color = lerp_color(tail_color, head_color, k);
            let (x, y) = pts[i];
            let r = 6.0 + 3.5 * k;
            draw_circle(x, y, r, color);
            if i + 1 < pts.len() {
                let (px, py) = pts[i + 1];
                draw_line(x, y, px, py, r * 1.6, color);
            }
        }

        // Голова и глаза
        let (hx, hy) = pts[0];
        let (dx, dy) = (dir.0 as f32, dir.1 as f32);
        let (nx, ny) = (-dy, dx);
        for side in [-1.0, 1.0] {
            let ex = hx + dx * 3.0 + nx * 4.5 * side;
            let ey = hy + dy * 3.0 + ny * 4.5 * side;
            draw_circle(ex, ey, 3.2, WHITE);
            draw_circle(ex + dx * 1.2, ey + dy * 1.2, 1.7, BLACK);
        }

        // Верхняя панель
        draw_rectangle(0.0, 0.0, screen_width(), HEADER, Color::from_rgba(25, 30, 40, 255));
        draw_text(&format!("Score: {}", score), 15.0, 28.0, 30.0, WHITE);
        let best_text = format!("Best: {}", best);
        let bw = measure_text(&best_text, None, 30, 1.0).width;
        draw_text(&best_text, screen_width() - bw - 15.0, 28.0, 30.0, GOLD);

        // Рамка вокруг поля
        draw_rectangle_lines(
            0.0,
            HEADER,
            COLS as f32 * CELL,
            ROWS as f32 * CELL,
            4.0,
            Color::from_rgba(25, 30, 40, 255),
        );

        // Экран проигрыша
        if game_over {
            draw_rectangle(
                0.0,
                HEADER,
                screen_width(),
                ROWS as f32 * CELL,
                Color::new(0.0, 0.0, 0.0, 0.6),
            );
            draw_centered("GAME OVER", 210.0, 60, Color::from_rgba(255, 80, 80, 255));
            draw_centered(&format!("Score: {}", score), 250.0, 34, WHITE);
            draw_centered("Press Enter to restart", 290.0, 28, LIGHTGRAY);
        }

        next_frame().await;
    }
}