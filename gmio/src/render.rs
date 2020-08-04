//! Game rendering functionality

mod opengl;

use crate::{atlas::AtlasBuilder, window::Window};
use serde::{Deserialize, Serialize};
use shared::types::Colour;
use std::any::Any;

// Re-export for more logical module pathing
pub use crate::atlas::AtlasRef;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedTexture {
    width: i32,
    height: i32,
    pixels: Box<[u8]>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BlendType {
    Zero,
    One,
    SrcColour,
    InvSrcColour,
    SrcAlpha,
    InvSrcAlpha,
    DestAlpha,
    InvDestAlpha,
    DestColour,
    InvDestColour,
    SrcAlphaSaturate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    PointList,
    LineList,
    LineStrip,
    LineLoop, // not used in GM8 but useful for drawing shapes
    TriList,
    TriStrip,
    TriFan,
}

pub struct Renderer(Box<dyn RendererTrait>);

pub trait RendererTrait {
    fn as_any(&self) -> &dyn Any;
    fn max_texture_size(&self) -> u32;
    fn push_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String>;
    fn upload_sprite(
        &mut self,
        data: Box<[u8]>,
        width: i32,
        height: i32,
        origin_x: i32,
        origin_y: i32,
    ) -> Result<AtlasRef, String>;
    fn duplicate_sprite(&mut self, atlas_ref: &AtlasRef) -> Result<AtlasRef, String>;
    fn delete_sprite(&mut self, atlas_ref: AtlasRef);

    fn set_vsync(&self, vsync: bool);
    fn get_vsync(&self) -> bool;
    fn wait_vsync(&self);

    fn draw_sprite(&mut self, tex: &AtlasRef, x: f64, y: f64, xs: f64, ys: f64, ang: f64, col: i32, alpha: f64) {
        self.draw_sprite_general(
            tex,
            0.0,
            0.0,
            tex.w.into(),
            tex.h.into(),
            x,
            y,
            xs,
            ys,
            ang,
            col,
            col,
            col,
            col,
            alpha,
        );
    }
    fn draw_sprite_general(
        &mut self,
        texture: &AtlasRef,
        part_x: f64,
        part_y: f64,
        part_w: f64,
        part_h: f64,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        col1: i32,
        col2: i32,
        col3: i32,
        col4: i32,
        alpha: f64,
    );
    fn set_view_matrix(&mut self, view: [f32; 16]);
    fn set_viewproj_matrix(&mut self, view: [f32; 16], proj: [f32; 16]);
    fn set_model_matrix(&mut self, model: [f32; 16]);
    fn mult_model_matrix(&mut self, model: [f32; 16]);
    fn set_projection_ortho(&mut self, x: f64, y: f64, w: f64, h: f64, angle: f64);
    fn set_view(
        &mut self,
        width: u32,
        height: u32,
        unscaled_width: u32,
        unscaled_height: u32,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
        src_angle: f64,
        port_x: i32,
        port_y: i32,
        port_w: i32,
        port_h: i32,
    );
    fn flush_queue(&mut self);
    fn present(&mut self);
    fn finish(&mut self, width: u32, height: u32, clear_colour: Colour);

    fn dump_sprite(&self, atlas_ref: &AtlasRef) -> Box<[u8]>;
    fn dump_sprite_part(&self, texture: &AtlasRef, part_x: i32, part_y: i32, part_w: i32, part_h: i32) -> Box<[u8]> {
        self.dump_sprite(&AtlasRef {
            atlas_id: texture.atlas_id,
            w: part_w,
            h: part_h,
            x: texture.x + part_x,
            y: texture.y + part_y,
            origin_x: 0.0,
            origin_y: 0.0,
        })
    }
    fn get_blend_mode(&self) -> (BlendType, BlendType);
    fn set_blend_mode(&mut self, src: BlendType, dst: BlendType);
    fn get_pixel_interpolation(&self) -> bool;
    fn set_pixel_interpolation(&mut self, lerping: bool);

    fn get_pixels(&self, x: i32, y: i32, w: i32, h: i32) -> Box<[u8]>;
    fn draw_raw_frame(&mut self, rgb: Box<[u8]>, w: i32, h: i32, clear_colour: Colour);

    fn dump_dynamic_textures(&self) -> Vec<Option<SavedTexture>>;
    fn upload_dynamic_textures(&mut self, textures: &[Option<SavedTexture>]);

    fn create_surface(&mut self, w: i32, h: i32) -> Result<AtlasRef, String>;
    fn set_target(&mut self, atlas_ref: &AtlasRef);
    fn reset_target(&mut self, w: i32, h: i32, unscaled_w: i32, unscaled_h: i32);

    fn draw_sprite_partial(
        &mut self,
        texture: &AtlasRef,
        part_x: f64,
        part_y: f64,
        part_w: f64,
        part_h: f64,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        self.draw_sprite_general(
            texture, part_x, part_y, part_w, part_h, x, y, xscale, yscale, angle, colour, colour, colour, colour, alpha,
        )
    }
    fn draw_sprite_tiled(
        &mut self,
        texture: &AtlasRef,
        mut x: f64,
        mut y: f64,
        xscale: f64,
        yscale: f64,
        colour: i32,
        alpha: f64,
        tile_end_x: Option<f64>,
        tile_end_y: Option<f64>,
    ) {
        let width = f64::from(texture.w) * xscale;
        let height = f64::from(texture.h) * yscale;

        if tile_end_x.is_some() {
            x = x.rem_euclid(width);
            if x > 0.0 {
                x -= width;
            }
        }
        if tile_end_y.is_some() {
            y = y.rem_euclid(height);
            if y > 0.0 {
                y -= height;
            }
        }

        let start_x = x;

        loop {
            loop {
                self.draw_sprite(texture, x, y, xscale, yscale, 0.0, colour, alpha);
                x += width;
                match tile_end_x {
                    Some(end_x) if x < end_x => (),
                    _ => break,
                }
            }
            x = start_x;
            y += height;
            match tile_end_y {
                Some(end_y) if y < end_y => (),
                _ => break,
            }
        }
    }

    fn draw_rectangle(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, colour: i32, alpha: f64);
    fn draw_rectangle_outline(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, colour: i32, alpha: f64);
    fn draw_rectangle_gradient(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        c1: i32,
        c2: i32,
        c3: i32,
        c4: i32,
        alpha: f64,
        outline: bool,
    );
    fn draw_point(&mut self, x: f64, y: f64, colour: i32, alpha: f64);
    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, width: Option<f64>, c1: i32, c2: i32, alpha: f64);
    fn draw_triangle(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
        c1: i32,
        c2: i32,
        c3: i32,
        alpha: f64,
        outline: bool,
    );
    fn draw_ellipse(&mut self, x: f64, y: f64, rad_x: f64, rad_y: f64, c1: i32, c2: i32, alpha: f64, outline: bool);
    fn draw_roundrect(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c1: i32, c2: i32, alpha: f64, outline: bool);
    fn set_circle_precision(&mut self, prec: i32);
    fn get_circle_precision(&self) -> i32;
    fn clear_view(&mut self, colour: Colour, alpha: f64);
}

pub struct RendererOptions {
    pub size: (u32, u32),
    pub vsync: bool,
    pub interpolate_pixels: bool,
}

impl Renderer {
    pub fn new(backend: (), options: &RendererOptions, window: &Window, clear_colour: Colour) -> Result<Self, String> {
        Ok(Self(Box::new(match backend {
            () => opengl::RendererImpl::new(options, window, clear_colour)?,
        })))
    }

    pub fn max_texture_size(&self) -> u32 {
        self.0.max_texture_size()
    }

    pub fn push_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String> {
        self.0.push_atlases(atl)
    }

    pub fn upload_sprite(
        &mut self,
        data: Box<[u8]>,
        width: i32,
        height: i32,
        origin_x: i32,
        origin_y: i32,
    ) -> Result<AtlasRef, String> {
        self.0.upload_sprite(data, width, height, origin_x, origin_y)
    }

    pub fn duplicate_sprite(&mut self, atlas_ref: &AtlasRef) -> Result<AtlasRef, String> {
        self.0.duplicate_sprite(atlas_ref)
    }

    pub fn delete_sprite(&mut self, atlas_ref: AtlasRef) {
        self.0.delete_sprite(atlas_ref)
    }

    pub fn set_vsync(&self, vsync: bool) {
        self.0.set_vsync(vsync)
    }

    pub fn get_vsync(&self) -> bool {
        self.0.get_vsync()
    }

    pub fn wait_vsync(&self) {
        self.0.wait_vsync()
    }

    pub fn draw_sprite(
        &mut self,
        texture: &AtlasRef,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        self.0.draw_sprite(texture, x, y, xscale, yscale, angle, colour, alpha)
    }

    pub fn draw_sprite_general(
        &mut self,
        texture: &AtlasRef,
        part_x: f64,
        part_y: f64,
        part_w: f64,
        part_h: f64,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        col1: i32,
        col2: i32,
        col3: i32,
        col4: i32,
        alpha: f64,
    ) {
        self.0.draw_sprite_general(
            texture, part_x, part_y, part_w, part_h, x, y, xscale, yscale, angle, col1, col2, col3, col4, alpha,
        )
    }

    pub fn set_view_matrix(&mut self, view: [f32; 16]) {
        self.0.set_view_matrix(view)
    }

    pub fn set_viewproj_matrix(&mut self, view: [f32; 16], proj: [f32; 16]) {
        self.0.set_viewproj_matrix(view, proj)
    }

    pub fn set_model_matrix(&mut self, model: [f32; 16]) {
        self.0.set_model_matrix(model)
    }

    pub fn mult_model_matrix(&mut self, model: [f32; 16]) {
        self.0.mult_model_matrix(model)
    }

    pub fn set_projection_ortho(&mut self, x: f64, y: f64, w: f64, h: f64, angle: f64) {
        self.0.set_projection_ortho(x, y, w, h, angle)
    }

    pub fn set_view(
        &mut self,
        width: u32,
        height: u32,
        unscaled_width: u32,
        unscaled_height: u32,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
        src_angle: f64,
        port_x: i32,
        port_y: i32,
        port_w: i32,
        port_h: i32,
    ) {
        self.0.set_view(
            width,
            height,
            unscaled_width,
            unscaled_height,
            src_x,
            src_y,
            src_w,
            src_h,
            src_angle,
            port_x,
            port_y,
            port_w,
            port_h,
        )
    }

    pub fn draw_sprite_partial(
        &mut self,
        texture: &AtlasRef,
        part_x: f64,
        part_y: f64,
        part_w: f64,
        part_h: f64,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        self.0.draw_sprite_partial(texture, part_x, part_y, part_w, part_h, x, y, xscale, yscale, angle, colour, alpha)
    }

    pub fn draw_sprite_tiled(
        &mut self,
        texture: &AtlasRef,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        colour: i32,
        alpha: f64,
        tile_end_x: Option<f64>,
        tile_end_y: Option<f64>,
    ) {
        self.0.draw_sprite_tiled(texture, x, y, xscale, yscale, colour, alpha, tile_end_x, tile_end_y)
    }

    pub fn draw_rectangle(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, colour: i32, alpha: f64) {
        self.0.draw_rectangle(x1, y1, x2, y2, colour, alpha)
    }

    pub fn draw_rectangle_outline(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, colour: i32, alpha: f64) {
        self.0.draw_rectangle_outline(x1, y1, x2, y2, colour, alpha)
    }

    pub fn draw_rectangle_gradient(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        c1: i32,
        c2: i32,
        c3: i32,
        c4: i32,
        alpha: f64,
        outline: bool,
    ) {
        self.0.draw_rectangle_gradient(x1, y1, x2, y2, c1, c2, c3, c4, alpha, outline)
    }

    pub fn draw_point(&mut self, x: f64, y: f64, colour: i32, alpha: f64) {
        self.0.draw_point(x, y, colour, alpha)
    }

    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, width: Option<f64>, c1: i32, c2: i32, alpha: f64) {
        self.0.draw_line(x1, y1, x2, y2, width, c1, c2, alpha)
    }

    pub fn draw_triangle(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
        c1: i32,
        c2: i32,
        c3: i32,
        alpha: f64,
        outline: bool,
    ) {
        self.0.draw_triangle(x1, y1, x2, y2, x3, y3, c1, c2, c3, alpha, outline)
    }

    pub fn draw_ellipse(
        &mut self,
        x: f64,
        y: f64,
        rad_x: f64,
        rad_y: f64,
        c1: i32,
        c2: i32,
        alpha: f64,
        outline: bool,
    ) {
        self.0.draw_ellipse(x, y, rad_x, rad_y, c1, c2, alpha, outline)
    }

    pub fn draw_roundrect(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c1: i32, c2: i32, alpha: f64, outline: bool) {
        self.0.draw_roundrect(x1, y1, x2, y2, c1, c2, alpha, outline)
    }

    pub fn set_circle_precision(&mut self, prec: i32) {
        self.0.set_circle_precision(prec)
    }

    pub fn get_circle_precision(&self) -> i32 {
        self.0.get_circle_precision()
    }

    pub fn dump_sprite(&self, atlas_ref: &AtlasRef) -> Box<[u8]> {
        self.0.dump_sprite(atlas_ref)
    }

    pub fn dump_sprite_part(
        &self,
        texture: &AtlasRef,
        part_x: i32,
        part_y: i32,
        part_w: i32,
        part_h: i32,
    ) -> Box<[u8]> {
        self.0.dump_sprite_part(texture, part_x, part_y, part_w, part_h)
    }

    pub fn get_pixels(&self, x: i32, y: i32, w: i32, h: i32) -> Box<[u8]> {
        self.0.get_pixels(x, y, w, h)
    }

    pub fn draw_raw_frame(&mut self, rgb: Box<[u8]>, w: i32, h: i32, clear_colour: Colour) {
        self.0.draw_raw_frame(rgb, w, h, clear_colour)
    }

    pub fn dump_dynamic_textures(&self) -> Vec<Option<SavedTexture>> {
        self.0.dump_dynamic_textures()
    }

    pub fn upload_dynamic_textures(&mut self, textures: &[Option<SavedTexture>]) {
        self.0.upload_dynamic_textures(textures)
    }

    pub fn create_surface(&mut self, w: i32, h: i32) -> Result<AtlasRef, String> {
        self.0.create_surface(w, h)
    }

    pub fn set_target(&mut self, atlas_ref: &AtlasRef) {
        self.0.set_target(atlas_ref)
    }

    pub fn reset_target(&mut self, w: i32, h: i32, unscaled_w: i32, unscaled_h: i32) {
        self.0.reset_target(w, h, unscaled_w, unscaled_h)
    }

    pub fn get_blend_mode(&self) -> (BlendType, BlendType) {
        self.0.get_blend_mode()
    }

    pub fn set_blend_mode(&mut self, src: BlendType, dst: BlendType) {
        self.0.set_blend_mode(src, dst)
    }

    pub fn get_pixel_interpolation(&self) -> bool {
        self.0.get_pixel_interpolation()
    }

    pub fn set_pixel_interpolation(&mut self, lerping: bool) {
        self.0.set_pixel_interpolation(lerping)
    }

    pub fn flush_queue(&mut self) {
        self.0.flush_queue()
    }

    pub fn clear_view(&mut self, colour: Colour, alpha: f64) {
        self.0.clear_view(colour, alpha)
    }

    pub fn present(&mut self) {
        self.0.present()
    }

    pub fn finish(&mut self, width: u32, height: u32, clear_colour: Colour) {
        self.0.finish(width, height, clear_colour)
    }
}

/// Multiply two mat4's together
fn mat4mult(m1: [f32; 16], m2: [f32; 16]) -> [f32; 16] {
    [
        (m1[0] * m2[0]) + (m1[1] * m2[4]) + (m1[2] * m2[8]) + (m1[3] * m2[12]),
        (m1[0] * m2[1]) + (m1[1] * m2[5]) + (m1[2] * m2[9]) + (m1[3] * m2[13]),
        (m1[0] * m2[2]) + (m1[1] * m2[6]) + (m1[2] * m2[10]) + (m1[3] * m2[14]),
        (m1[0] * m2[3]) + (m1[1] * m2[7]) + (m1[2] * m2[11]) + (m1[3] * m2[15]),
        (m1[4] * m2[0]) + (m1[5] * m2[4]) + (m1[6] * m2[8]) + (m1[7] * m2[12]),
        (m1[4] * m2[1]) + (m1[5] * m2[5]) + (m1[6] * m2[9]) + (m1[7] * m2[13]),
        (m1[4] * m2[2]) + (m1[5] * m2[6]) + (m1[6] * m2[10]) + (m1[7] * m2[14]),
        (m1[4] * m2[3]) + (m1[5] * m2[7]) + (m1[6] * m2[11]) + (m1[7] * m2[15]),
        (m1[8] * m2[0]) + (m1[9] * m2[4]) + (m1[10] * m2[8]) + (m1[11] * m2[12]),
        (m1[8] * m2[1]) + (m1[9] * m2[5]) + (m1[10] * m2[9]) + (m1[11] * m2[13]),
        (m1[8] * m2[2]) + (m1[9] * m2[6]) + (m1[10] * m2[10]) + (m1[11] * m2[14]),
        (m1[8] * m2[3]) + (m1[9] * m2[7]) + (m1[10] * m2[11]) + (m1[11] * m2[15]),
        (m1[12] * m2[0]) + (m1[13] * m2[4]) + (m1[14] * m2[8]) + (m1[15] * m2[12]),
        (m1[12] * m2[1]) + (m1[13] * m2[5]) + (m1[14] * m2[9]) + (m1[15] * m2[13]),
        (m1[12] * m2[2]) + (m1[13] * m2[6]) + (m1[14] * m2[10]) + (m1[15] * m2[14]),
        (m1[12] * m2[3]) + (m1[13] * m2[7]) + (m1[14] * m2[11]) + (m1[15] * m2[15]),
    ]
}
