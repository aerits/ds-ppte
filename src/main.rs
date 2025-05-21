use std::fs::File;
use std::io::{self, Write};
use std::mem::MaybeUninit;
use std::ptr::{self, null, null_mut};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use blockstackers_core::blockstacker::{self, BlockStacker, Tuning};
use blockstackers_core::buyo_game::BuyoBuyo;
use blockstackers_core::randomizer::Randomizer;
use blockstackers_core::tet::Tet;
use blockstackers_core::Sprite;
use citro2d_sys::*;
use citro3d_sys::*;
use ctru::prelude::*;
use ctru::services::gfx::{Flush, RawFrameBuffer, Screen, Swap};
use rand::Rng;

mod menu;

fn main() {
    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let _console = Console::new(gfx.bottom_screen.borrow_mut());

    println!("i am nintendo..");

    let mut rng = rand::rng();

    let mut circle_pos = (200, 120);
    let render_target = unsafe {
        C3D_Init(C3D_DEFAULT_CMDBUF_SIZE as usize);
        C2D_Init(C2D_DEFAULT_MAX_OBJECTS.into());
        C2D_Prepare();
        C2D_CreateScreenTarget(ctru_sys::GFX_TOP, ctru_sys::GFX_LEFT)
    };

    let color = unsafe { C2D_Color32(255, 255, 255, 0) };
    // let i: i8 = rng.random();
    // let game;
    // if i >= 0 {
    //     game = "tet"
    // } else {
    //     game = "buyo"
    // }
    let mut game: Box<dyn BlockStacker> = <dyn BlockStacker>::new(
        "buyo",
        6,
        12,
        Randomizer::new(
            7,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        ),
        Tuning::new(),
    );
    let mut pieces_placed = 1;
    let time_start = Instant::now();
    let mut time_last_update = Instant::now();

    let buf = unsafe { C2D_TextBufNew(4096) };
    let mut text: MaybeUninit<C2D_Text> = MaybeUninit::uninit();

    while apt.main_loop() {
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }
        let mut update = false;

        if hid.keys_held().contains(KeyPad::DPAD_DOWN) {
            circle_pos.1 += 10; update = true;
        }
        if hid.keys_down().contains(KeyPad::DPAD_UP) {
            circle_pos.1 -= 10; update = true;
            game.hard_drop();
            time_last_update = Instant::now();
        }
        if hid.keys_down().contains(KeyPad::DPAD_LEFT) {
            circle_pos.0 -= 10; update = true;
            game.input_left();
        }
        if hid.keys_down().contains(KeyPad::DPAD_RIGHT) {
            circle_pos.0 += 10; update = true;
            game.input_right();
        }

        if hid.keys_down().contains(KeyPad::A) {
            // color = unsafe { C2D_Color32(rng.random(), rng.random(), rng.random(), 0) };
            game.input_rotation_left(); update = true;
        }
        if hid.keys_down().contains(KeyPad::B) {
            game.input_rotation_right(); update = true;
        }

        if game.game_loop(
            time_last_update.duration_since(time_start).as_millis() as u64,
            Instant::now().duration_since(time_start).as_millis() as u64,
        ) { 
            time_last_update = Instant::now();
            update = true;
            if game.get_loop_state() == "SpawnNew" {
                pieces_placed += 1;
                
            } else if game.get_loop_state() == "Dead" {
                sleep(Duration::from_secs(5));
                game = <dyn BlockStacker>::new(
                    "buyo",
                    6,
                    12,
                    Randomizer::new(
                        7,
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                    ),
                    Tuning::new(),
                );
            }
            let score = game.total_score();
            let tuning = game.get_mut_tuning();
            tuning.fall_speed = 100;
            tuning.fall_skip = 0.2 + pieces_placed as f32 / 100.0;
        }

        if !update {
            continue;
        }

        
        unsafe {
            println!("\x1b[14;00H");
            println!("{}, {}", C3D_GetDrawingTime(), C3D_GetProcessingTime());
            println!("level: {}", pieces_placed);
            // println!("fall skip: {}", 0.2 + (game.total_score() as f32 / 100f32) % 10.0);
            C3D_FrameBegin(C3D_FRAME_SYNCDRAW);
            C2D_TargetClear(render_target, color);
            C2D_SceneBegin(render_target);

            C2D_TextBufClear(buf);

            C2D_TextParse(
                text.as_mut_ptr(),
                buf,
                (game.total_score().to_string() + "\nlevel: " + &pieces_placed.to_string() + "\0").as_ptr(),
            );
            C2D_TextOptimize(text.as_mut_ptr());
            C2D_DrawText(text.as_ptr(), 0, 120.0, 30.0, 0.0, 1.0, 1.0);
            const RADIUS: f32 = 8.0;
            for (v, s) in game.get_board() {
                let color = get_sprite(s);
                C2D_DrawCircleSolid(v.x as f32 * 2.0 * RADIUS, v.y as f32 * 2.0 * RADIUS, 0f32, RADIUS, color);
            }
            for (x, y, s) in game.get_controlled_block() {
                let color = get_sprite(s);
                C2D_DrawCircleSolid(x as f32 * 2.0 * RADIUS, y as f32 * 2.0 * RADIUS, 0f32, RADIUS, color);
            }
            for (v, s) in game.next_queue() {
                let color = get_sprite(s);
                C2D_DrawCircleSolid(v.x as f32 * 2.0 * RADIUS + 200.0, v.y as f32 * 2.0 * RADIUS + 100.0, 0f32, RADIUS, color);
            }

            // println!("{}", C3D_GetDrawingTime());
            C3D_FrameEnd(0);
        }

        gfx.wait_for_vblank();
    }
    unsafe {
        C2D_TextBufDelete(buf);
    }
}

fn get_sprite(s: Sprite) -> ctru_sys::u32_ {
    
    unsafe {
        let red = C2D_Color32(255, 0, 0, 255);
        let blue = C2D_Color32(0, 0, 255, 255);
        let yellow = C2D_Color32(255, 255, 0, 255);
        let purple = C2D_Color32(255, 0, 255, 255);
        let green = C2D_Color32(0, 255, 0, 255);
        match s {
            Sprite::Wall => C2D_Color32(0, 0, 0, 255),
            Sprite::BuyoRed => red,
            Sprite::BuyoBlue => blue,
            Sprite::BuyoYellow => yellow,
            Sprite::BuyoPurple => purple,
            Sprite::BuyoGreen => green,
            Sprite::TetT => purple,
            Sprite::TetI => blue,
            Sprite::TetO => yellow,
            Sprite::TetJ => blue,
            Sprite::TetL => C2D_Color32(255, 165, 0, 255),
            Sprite::TetS => red,
            Sprite::TetZ => green,
        }
    }
}