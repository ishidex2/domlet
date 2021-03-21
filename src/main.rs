mod atoms;
mod bucket_array;
mod ui;
mod xml_ui;
extern crate sdl2;
extern crate ron;
extern crate xml;
use crate::atoms::*;
use crate::bucket_array::*;

use serde::{Deserialize, Serialize};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::collections::hash_map::*;

#[derive(Debug, Deserialize)]
struct TexRect(u32, u32, u32, u32);

#[derive(Debug, Deserialize)]
struct PositionInfo {
    texture_rect: TexRect
}

#[derive(Deserialize)]
struct Config {
    window_size: (u32, u32)
}

#[derive(Deserialize, Serialize)]
struct SaveData {
    name: String,
    login_count: u32
}

type SRect = sdl2::rect::Rect;

pub struct RenderData
{
    pub cutout: Option<SRect>
}


pub struct FontData
{
    pub bold: bool,
    pub italic: bool,
    pub letter_spacing: i32,
    pub row_spacing: i32,
    pub color: (u8, u8, u8, u8)
}

pub fn render_copy_cut(canv: &mut sdl2::render::WindowCanvas, tex: &sdl2::render::Texture, src: SRect, dest: SRect, cutout: Option<SRect>)
{
    canv.set_clip_rect(cutout);
    canv.copy(&tex, src, dest).unwrap();
    canv.set_clip_rect(None);
}


pub fn render_text(canv: &mut sdl2::render::WindowCanvas, tex: &mut sdl2::render::Texture, text: &str, pos: Vec2<i32>, dat: &FontData, ren: &RenderData)
{
    let mut x = 0;
    let mut y = 0;
    for chr in text.chars()
    {
        if chr == '\n'
        {
            y += dat.row_spacing;
            x = 0;
            continue;
        }
        if chr == '\t'
        {
            x += 8*4;
            continue;
        }
        let i = if (chr as u8) >= 32 && (chr as u8) < 128 {chr as u32 - 31} else { 0 };
        tex.set_color_mod(dat.color.0, dat.color.1, dat.color.2);
        let sox = (i*8)%512;
        let soy = ((i*8)/512)*16;
        let its = if dat.bold { 2 } else { 1 };
        let crop = if dat.italic { 9 } else { 16 };
        for it in 0..its
        {
            render_copy_cut(canv, &tex, SRect::new(sox as i32, soy as i32, 8, crop), SRect::new(pos.x+x+it + if dat.italic { 1 } else { 0 }, pos.y+y, 8, crop), ren.cutout);
            if dat.italic
            {
                render_copy_cut(canv, &tex, SRect::new(sox as i32, soy as i32+crop as i32, 8, 16-crop), SRect::new(pos.x+x+it, pos.y+crop as i32+y, 8, 16-crop), ren.cutout);
            }
        }
        
        x += dat.letter_spacing;
    }
}


pub fn main() {
    let mut ui = xml_ui::parse_xml("./test.xml");

    ui.propagate_styles_rec(ui.root, HashMap::new());

    use std::fs::read_to_string;
    let config: Config = ron::from_str(
        &read_to_string("./config.ron").unwrap()
    ).map_err(|e| e.to_string()).unwrap();
    let mut save_data: SaveData = ron::from_str(
        &read_to_string("./savedata.ron").unwrap()
    ).map_err(|e| e.to_string()).unwrap();

    save_data.login_count += 1;

    std::fs::write("./savedata.ron", ron::to_string(&save_data).unwrap()).unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", config.window_size.0*2, config.window_size.1*2)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let creator = canvas.texture_creator();
    
    let mut mouse_pos = (0, 0);
    let font_img = sdl2::surface::Surface::load_bmp("font.bmp").unwrap();
    let mut font_tex = font_img.as_texture(&creator).or(Err("Can't convert to texture")).unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut err: Option<String> = None;
    let mut frames = ui.calculate_layout();
    frames.reverse();

    let mut i = 0;
    'running: loop {
        let si = (f32::sin((i as f32)/255.*std::f32::consts::TAU)*128.0+128.) as u8;
        canvas.set_draw_color(Color::RGB(si/16, 0, 255/16 - si/16));
        canvas.clear();
        let wsize = canvas.window().size();
        let out = format!("Hello {}, {} logins", save_data.name, save_data.login_count);

        canvas.string((wsize.0-(&out.len()*8) as u32) as i16, wsize.1 as i16-8, &out, Color::BLACK);
        canvas.string((wsize.0-(&out.len()*8) as u32-1) as i16, wsize.1 as i16-9, &out, Color::WHITE);

        if let Some(text) = &err {
            canvas.string(0, wsize.1 as i16-8, &text, Color::BLACK);
            canvas.string(-1, wsize.1 as i16-9, &text, Color::WHITE);
        }


        for event in event_pump.poll_iter() {
            match event {
                Event::MouseMotion { x, y, .. } => {
                    mouse_pos.0 = x;
                    mouse_pos.1 = y;
                },
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    err = None;
                    ui = xml_ui::parse_xml("./test.xml");
                    ui.propagate_styles_rec(ui.root, HashMap::new());
                    frames = ui.calculate_layout();
                    frames.reverse();
                },
                _ => {}
            }
        }
        for i in frames.iter() {
            let elem = &match ui.things.get(i.for_id) { None => break, Some(x) => x };
            let border = elem.get_border();
            canvas.set_draw_color(Color::RGBA(border.1.0, border.1.1, border.1.2, border.1.3));
            //canvas.fill_rect(sdl2::rect::Rect::new(i.rect.pos.x as i32-border.0 as i32, i.rect.pos.y as i32-border.0 as i32, i.rect.size.x as u32+(border.0*2.) as u32, i.rect.size.y as u32+(border.0*2.) as u32));
            let offs = (wsize.0 as i32/2-frames[0].rect.size.x as i32/2, wsize.1 as i32/2-frames[0].rect.size.y as i32/2);
            canvas.fill_rect(sdl2::rect::Rect::new(offs.0+i.rect.pos.x as i32-border.0 as i32, offs.1+i.rect.pos.y as i32-border.0 as i32, i.rect.size.x as u32+(border.0*1.) as u32, border.0 as u32));
            canvas.fill_rect(sdl2::rect::Rect::new(offs.0+i.rect.pos.x as i32-border.0 as i32, offs.1+i.rect.pos.y as i32, border.0 as u32, i.rect.size.y as u32+(border.0*1.) as u32));
            canvas.fill_rect(sdl2::rect::Rect::new(offs.0+i.rect.pos.x as i32 as i32, offs.1+i.rect.pos.y as i32+i.rect.size.y as i32, i.rect.size.x as u32+(border.0*1.) as u32, border.0 as u32));
            canvas.fill_rect(sdl2::rect::Rect::new(offs.0+i.rect.pos.x as i32+i.rect.size.x as i32, offs.1+i.rect.pos.y as i32-border.0 as i32, border.0 as u32, i.rect.size.y as u32+(border.0*1.) as u32));
            match &elem.elem {
                ui::Ui::Text { text } => {
                    let color = elem.get_fg();
                    render_text(&mut canvas, &mut font_tex, &text, Vec2::new(i.rect.pos.x as i32+offs.0, i.rect.pos.y as i32+offs.1), &FontData {bold: true, italic: false, letter_spacing: 8, row_spacing: 16, color}, &RenderData { cutout: None });
                },
                ui::Ui::Button => {
                    canvas.set_draw_color(Color::RED);
                    canvas.fill_rect(sdl2::rect::Rect::new(i.rect.pos.x as i32+offs.0, offs.1+i.rect.pos.y as i32, i.rect.size.x as u32, i.rect.size.y as u32));
                },
                ui::Ui::Div => {
                    let color = elem.get_bg();
                    canvas.set_draw_color(Color::RGBA(color.0, color.1, color.2, color.3));
                    canvas.fill_rect(sdl2::rect::Rect::new(i.rect.pos.x as i32+offs.0, offs.1+i.rect.pos.y as i32, i.rect.size.x as u32, i.rect.size.y as u32));
                },
                _ => {}
            }
        };

        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
