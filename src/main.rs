use std::fs::File;
use std::io::{self, Write};
use std::mem::MaybeUninit;
use std::ptr::{self, null, null_mut};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use blockstackers_core::blockstacker::{BlockStacker, Tuning};
use blockstackers_core::buyo_game::BuyoBuyo;
use blockstackers_core::randomizer::Randomizer;
use blockstackers_core::tet::Tet;
use citro2d_sys::*;
use citro3d_sys::*;
use ctru::prelude::*;
use ctru::services::gfx::{Flush, RawFrameBuffer, Screen, Swap};
use rand::Rng;

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

    let mut color = unsafe { C2D_Color32(255, 255, 255, 0) };

    let mut game = BuyoBuyo::new(
        6,
        12,
        Randomizer::new(
            4,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        ),
        Tuning::new(),
    );
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
        }

        if !update {
            continue;
        }

        
        unsafe {
            println!("\x1b[14;00H");
            println!("{}, {}", C3D_GetDrawingTime(), C3D_GetProcessingTime());
            C3D_FrameBegin(C3D_FRAME_SYNCDRAW);
            C2D_TargetClear(render_target, color);
            C2D_SceneBegin(render_target);

            C2D_TextBufClear(buf);

            C2D_TextParse(
                text.as_mut_ptr(),
                buf,
                (game.total_score().to_string() + "\0").as_ptr(),
            );
            C2D_TextOptimize(text.as_mut_ptr());
            C2D_DrawText(text.as_ptr(), 0, 120.0, 30.0, 0.0, 1.0, 1.0);

            for (v, s) in game.get_board() {
                let color = match s {
                    blockstackers_core::Sprite::Wall => C2D_Color32(0, 0, 0, 255),
                    blockstackers_core::Sprite::BuyoRed => C2D_Color32(255, 0, 0, 255),
                    blockstackers_core::Sprite::BuyoBlue => C2D_Color32(0, 0, 255, 255),
                    blockstackers_core::Sprite::BuyoYellow => C2D_Color32(255, 255, 0, 255),
                    blockstackers_core::Sprite::BuyoPurple => C2D_Color32(255, 0, 255, 255),
                    blockstackers_core::Sprite::BuyoGreen => C2D_Color32(0, 255, 0, 255),
                    _ => panic!(),
                };
                C2D_DrawCircleSolid(v.x as f32 * 16f32, v.y as f32 * 16f32, 0f32, 8f32, color);
            }
            for (x, y, s) in game.get_controlled_block() {
                let color = match s {
                    blockstackers_core::Sprite::Wall => C2D_Color32(0, 0, 0, 255),
                    blockstackers_core::Sprite::BuyoRed => C2D_Color32(255, 0, 0, 255),
                    blockstackers_core::Sprite::BuyoBlue => C2D_Color32(0, 0, 255, 255),
                    blockstackers_core::Sprite::BuyoYellow => C2D_Color32(255, 255, 0, 255),
                    blockstackers_core::Sprite::BuyoPurple => C2D_Color32(255, 0, 255, 255),
                    blockstackers_core::Sprite::BuyoGreen => C2D_Color32(0, 255, 0, 255),
                    _ => panic!(),
                };
                C2D_DrawCircleSolid(x as f32 * 16f32, y as f32 * 16f32, 0f32, 8f32, color);
            }
            for (v, s) in game.next_queue() {
                let color = match s {
                    blockstackers_core::Sprite::Wall => C2D_Color32(0, 0, 0, 255),
                    blockstackers_core::Sprite::BuyoRed => C2D_Color32(255, 0, 0, 255),
                    blockstackers_core::Sprite::BuyoBlue => C2D_Color32(0, 0, 255, 255),
                    blockstackers_core::Sprite::BuyoYellow => C2D_Color32(255, 255, 0, 255),
                    blockstackers_core::Sprite::BuyoPurple => C2D_Color32(255, 0, 255, 255),
                    blockstackers_core::Sprite::BuyoGreen => C2D_Color32(0, 255, 0, 255),
                    _ => panic!(),
                };
                C2D_DrawCircleSolid(v.x as f32 * 16f32 + 200.0, v.y as f32 * 16f32 + 100.0, 0f32, 8f32, color);
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
