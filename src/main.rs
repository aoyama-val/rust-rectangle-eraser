use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime};
mod model;
use crate::model::*;

pub const SCREEN_WIDTH: i32 = CELL_SIZE * FIELD_W as i32;
pub const SCREEN_HEIGHT: i32 = CELL_SIZE * 16;

struct Image<'a> {
    texture: Texture<'a>,
    #[allow(dead_code)]
    w: u32,
    h: u32,
}

impl<'a> Image<'a> {
    fn new(texture: Texture<'a>) -> Self {
        let q = texture.query();
        let image = Image {
            texture,
            w: q.width,
            h: q.height,
        };
        image
    }
}

struct Resources<'a> {
    images: HashMap<String, Image<'a>>,
    chunks: HashMap<String, sdl2::mixer::Chunk>,
    fonts: HashMap<String, sdl2::ttf::Font<'a, 'a>>,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rust-blackjack", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    sdl_context.mouse().show_cursor(false);

    init_mixer();

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);

    let texture_creator = canvas.texture_creator();
    let mut resources = load_resources(&texture_creator, &mut canvas, &ttf_context);

    let mut event_pump = sdl_context.event_pump()?;

    let mut game = Game::new(2, 2);

    println!("Keys:");
    println!("  Space       : Restart when gameover");

    'running: loop {
        let started = SystemTime::now();

        let mut command = Command::None;
        let mut is_keydown = false;

        let keyboard_state = event_pump.keyboard_state();
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left) {
            command = Command::Left;
        } else if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
            command = Command::Right;
        } else if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Space) {
            command = Command::Shoot;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    if code == Keycode::Escape {
                        break 'running;
                    }
                    is_keydown = true;
                    match code {
                        Keycode::Space => {
                            if game.is_over {
                                // game = Game::new();
                                // game.init();
                                // } else {
                                //     command = Command::Shoot;
                            }
                        }
                        Keycode::F1 => game.toggle_debug(),
                        _ => {}
                    };
                }
                _ => {}
            }
        }
        if !game.is_debug || is_keydown {
            game.update(command);
        }
        render(&mut canvas, &game, &mut resources)?;

        play_sounds(&mut game, &resources);

        if game.is_over {
            if game.height < game.width {
                game = Game::new(game.width, game.height + 1);
            } else {
                game = Game::new(game.width + 1, 2);
            }
        }

        let finished = SystemTime::now();
        let elapsed = finished.duration_since(started).unwrap();
        let frame_duration = Duration::new(0, 1_000_000_000u32 / model::FPS as u32);
        if elapsed < frame_duration {
            ::std::thread::sleep(frame_duration - elapsed)
        }
    }

    Ok(())
}

fn init_mixer() {
    let chunk_size = 1_024;
    mixer::open_audio(
        mixer::DEFAULT_FREQUENCY,
        mixer::DEFAULT_FORMAT,
        mixer::DEFAULT_CHANNELS,
        chunk_size,
    )
    .expect("cannot open audio");
    let _mixer_context = mixer::init(mixer::InitFlag::MP3).expect("cannot init mixer");
}

fn load_resources<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    #[allow(unused_variables)] canvas: &mut Canvas<Window>,
    ttf_context: &'a Sdl2TtfContext,
) -> Resources<'a> {
    let mut resources = Resources {
        images: HashMap::new(),
        chunks: HashMap::new(),
        fonts: HashMap::new(),
    };

    let entries = fs::read_dir("resources/image").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".bmp") {
            let temp_surface = sdl2::surface::Surface::load_bmp(&path).unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&temp_surface)
                .expect(&format!("cannot load image: {}", path_str));

            let basename = path.file_name().unwrap().to_str().unwrap();
            let image = Image::new(texture);
            resources.images.insert(basename.to_string(), image);
        }
    }

    let entries = fs::read_dir("./resources/sound").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".wav") {
            let chunk = mixer::Chunk::from_file(path_str)
                .expect(&format!("cannot load sound: {}", path_str));
            let basename = path.file_name().unwrap().to_str().unwrap();
            resources.chunks.insert(basename.to_string(), chunk);
        }
    }

    load_font(
        &mut resources,
        &ttf_context,
        "./resources/font/boxfont2.ttf",
        32,
        "boxfont",
    );

    resources
}

fn load_font<'a>(
    resources: &mut Resources<'a>,
    ttf_context: &'a Sdl2TtfContext,
    path_str: &str,
    point_size: u16,
    key: &str,
) {
    let font = ttf_context
        .load_font(path_str, point_size)
        .expect(&format!("cannot load font: {}", path_str));
    resources.fonts.insert(key.to_string(), font);
}

fn render(
    canvas: &mut Canvas<Window>,
    game: &Game,
    resources: &mut Resources,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let font = resources.fonts.get_mut("boxfont").unwrap();

    // render field
    for y in 0..FIELD_H {
        for x in 0..FIELD_W {
            let ch = game.field[y][x];
            if ch != ' ' {
                let color_index = (ch as i32) % 6;
                let color = match color_index {
                    1 => Color::RGB(255, 128, 128),
                    2 => Color::RGB(128, 255, 128),
                    3 => Color::RGB(128, 128, 255),
                    4 => Color::RGB(255, 255, 128),
                    5 => Color::RGB(128, 255, 255),
                    0 => Color::RGB(255, 128, 255),
                    _ => panic!("Invalid n: {}", ch),
                };
                canvas.set_draw_color(color);
                canvas.fill_rect(Rect::new(
                    x as i32 * CELL_SIZE,
                    y as i32 * CELL_SIZE,
                    CELL_SIZEu32,
                    CELL_SIZEu32,
                ))?;
            }
        }
    }

    // render player
    canvas.set_draw_color(Color::RGB(192, 192, 192));
    let offset_x;
    if game.move_wait > 0 {
        offset_x = ((if game.move_dir == Direction::Left {
            -1.0
        } else {
            1.0
        }) * ((MOVE_WAIT - game.move_wait) as f32 / MOVE_WAIT as f32)
            * CELL_SIZE as f32) as i32;
    } else {
        offset_x = 0;
    }
    canvas.fill_rect(Rect::new(
        game.player_x as i32 * CELL_SIZE + offset_x,
        SCREEN_HEIGHT - CELL_SIZE,
        CELL_SIZEu32,
        CELL_SIZEu32,
    ))?;

    // render bullets
    for bullet in &game.bullets {
        canvas.set_draw_color(Color::RGB(128, 128, 128));
        canvas.fill_rect(Rect::new(
            bullet.pos.x as i32 * CELL_SIZE,
            bullet.pos.y as i32 * CELL_SIZE + bullet.offset_y,
            CELL_SIZEu32,
            CELL_SIZEu32,
        ))?;
    }

    // 渦巻き状に消えるブロックを描画
    // canvas.set_draw_color(Color::RGB(255, 128, 128));
    // for y in 0..game.height {
    //     for x in 0..game.width {
    //         if !game.erased[y][x] {
    //             canvas.fill_rect(Rect::new(
    //                 100 + x as i32 * CELL_SIZE,
    //                 100 + y as i32 * CELL_SIZE,
    //                 CELL_SIZEu32 - 1,
    //                 CELL_SIZEu32 - 1,
    //             ))?;
    //         }
    //     }
    // }

    render_font(
        canvas,
        font,
        format!("{} x {}", game.width, game.height).to_string(),
        0,
        0,
        Color::RGBA(255, 255, 255, 255),
        false,
    );

    if game.is_over {
        render_font(
            canvas,
            font,
            "GAME OVER".to_string(),
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT / 2,
            Color::RGBA(128, 128, 255, 255),
            true,
        );
    }

    if game.is_clear {
        render_font(
            canvas,
            font,
            "CLEAR".to_string(),
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT / 2,
            Color::RGBA(255, 255, 128, 255),
            true,
        );
    }

    canvas.present();

    Ok(())
}

fn render_font(
    canvas: &mut Canvas<Window>,
    font: &sdl2::ttf::Font,
    text: String,
    x: i32,
    y: i32,
    color: Color,
    center: bool,
) {
    let texture_creator = canvas.texture_creator();

    let surface = font.render(&text).blended(color).unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let x: i32 = if center {
        x - texture.query().width as i32 / 2
    } else {
        x
    };
    canvas
        .copy(
            &texture,
            None,
            Rect::new(x, y, texture.query().width, texture.query().height),
        )
        .unwrap();
}

fn play_sounds(game: &mut Game, resources: &Resources) {
    for sound_key in &game.requested_sounds {
        let chunk = resources
            .chunks
            .get(&sound_key.to_string())
            .expect("cannot get sound");
        sdl2::mixer::Channel::all()
            .play(&chunk, 0)
            .expect("cannot play sound");
    }
    game.requested_sounds = Vec::new();
}
