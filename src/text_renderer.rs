use crate::essentials::Vec2;

type SRect = sdl2::rect::Rect;

pub struct RenderData {
    pub cutout: Option<SRect>,
}

pub struct FontData {
    pub bold: bool,
    pub italic: bool,
    pub letter_spacing: i32,
    pub row_spacing: i32,
    pub color: (u8, u8, u8, u8),
}

pub fn render_copy_cut(
    canv: &mut sdl2::render::WindowCanvas,
    tex: &sdl2::render::Texture,
    src: SRect,
    dest: SRect,
    cutout: Option<SRect>,
) {
    canv.set_clip_rect(cutout);
    canv.copy(&tex, src, dest).unwrap();
    canv.set_clip_rect(None);
}

pub fn render_text(
    canv: &mut sdl2::render::WindowCanvas,
    tex: &mut sdl2::render::Texture,
    text: &str,
    pos: Vec2<i32>,
    dat: &FontData,
    ren: &RenderData,
) {
    let mut x = 0;
    let mut y = 0;
    for chr in text.chars() {
        if chr == '\n' {
            y += dat.row_spacing;
            x = 0;
            continue;
        }
        if chr == '\t' {
            x += 8 * 4;
            continue;
        }
        let i = if (chr as u8) >= 32 && (chr as u8) < 128 {
            chr as u32 - 31
        } else {
            0
        };
        tex.set_alpha_mod(dat.color.3);
        tex.set_color_mod(dat.color.0, dat.color.1, dat.color.2);
        let sox = (i * 8) % 512;
        let soy = ((i * 8) / 512) * 16;
        let its = if dat.bold { 2 } else { 1 };
        let crop = if dat.italic { 9 } else { 16 };
        for it in 0..its {
            render_copy_cut(
                canv,
                &tex,
                SRect::new(sox as i32, soy as i32, 8, crop),
                SRect::new(
                    pos.x + x + it + if dat.italic { 1 } else { 0 },
                    pos.y + y,
                    8,
                    crop,
                ),
                ren.cutout,
            );
            if dat.italic {
                render_copy_cut(
                    canv,
                    &tex,
                    SRect::new(sox as i32, soy as i32 + crop as i32, 8, 16 - crop),
                    SRect::new(pos.x + x + it, pos.y + crop as i32 + y, 8, 16 - crop),
                    ren.cutout,
                );
            }
        }

        x += dat.letter_spacing;
    }
}
