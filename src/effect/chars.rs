use crate::{
    System,
    graphics::{CHAR_ATLAS_CHANNEL_COUNT, CHAR_ATLAS_SIZE},
    logs::*,
    res::FONT,
};
use ab_glyph::*;
use etagere::*;
use glam::*;
use lru::LruCache;

/// 1文字テクスチャの1辺におけるマージン
///
/// NOTE: 境界で隣の文字のピクセルを含まないように四辺に2pxのマージンを取る。
///       どうせadvanceを基準に詰めていくので、いくらマージンを取っても構わない。
const MARGIN: usize = 2;
/// 1文字テクスチャの1方向におけるマージン
const MARGIN_AXIS: usize = MARGIN * 2;

#[derive(Clone, Copy)]
pub struct CharInfo {
    alloc_id: AllocId,
    pub width: f32,
    pub height: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub advance: f32,
    pub uv: Vec4,
}

pub type CharLruCache = LruCache<(char, u32), CharInfo>;

pub struct CharsManageState {
    /// フォント
    ///
    /// HINT: 複数フォントに対応する場合はHashMapにして、
    ///       外部からフォント識別子を受け取るのが良いだろう。
    font: FontRef<'static>,
    /// 文字テクスチャアトラスに登録されている文字のUV範囲
    atlas: AtlasAllocator,
    /// 文字テクスチャアトラスに登録されている文字とそのID
    ///
    /// 文字テクスチャアトラスが満ちている場合に
    /// 最も古く使われた文字を消すためにLRUキャッシュを用いている。
    registereds: CharLruCache,
}

impl CharsManageState {
    pub fn new(system: &System) -> Self {
        // NOTE: 物理解像度と論理解像度の違いを考慮してスケールアップ。
        let atlas_size = (CHAR_ATLAS_SIZE as f32 * system.window.get_scale_factor()) as i32;
        Self {
            font: FontRef::try_from_slice(FONT).expect_log("failed to load a font"),
            atlas: AtlasAllocator::new(etagere::size2(atlas_size, atlas_size)),
            registereds: LruCache::unbounded(),
        }
    }
}

impl CharsManageState {
    // NOTE: `scale`は`f32`でしか使われないが、
    //       `f32`は`Hash`を実装していないので`u32`で受け取る。
    pub fn update(mut self, c: char, scale: u32, mut system: System) -> (Self, CharInfo, System) {
        if let Some(&info) = self.registereds.get(&(c, scale)) {
            (self, info, system)
        } else {
            (self, system) = self.register_character(c, scale, system);
            let info = *self.registereds.get(&(c, scale)).unwrap();
            (self, info, system)
        }
    }

    fn register_character(mut self, c: char, scale: u32, mut system: System) -> (Self, System) {
        let scale_factor = system.window.get_scale_factor();

        // ラスタライズ
        //
        // NOTE: 物理解像度と論理解像度の違いを考慮してスケールアップ。
        //       scaleをスケールアップすれば自ずとラスタライズ結果も比例してスケールアップされる。
        let rasterized = rasterize_character(&self.font, c, scale as f32 * scale_factor);

        // アロケート
        let size = etagere::size2(rasterized.width as i32, rasterized.height as i32);
        let allocated = loop {
            if let Some(alloc) = self.atlas.allocate(size) {
                break alloc;
            } else {
                let least_used = self
                    .registereds
                    .pop_lru()
                    .expect("Internal error: no character exists but character atlas is full");
                self.atlas.deallocate(least_used.1.alloc_id);
            }
        };

        // アップロード
        system.gengine = system.gengine.upload_char(
            &rasterized.data,
            allocated.rectangle.min.x,
            allocated.rectangle.min.y,
            rasterized.width,
            rasterized.height,
        );

        // スケールアップとマージンの除去
        let x = (allocated.rectangle.min.x as f32 + MARGIN as f32) / scale_factor;
        let y = (allocated.rectangle.min.y as f32 + MARGIN as f32) / scale_factor;
        let width = (rasterized.width as f32 - MARGIN_AXIS as f32) / scale_factor;
        let height = (rasterized.height as f32 - MARGIN_AXIS as f32) / scale_factor;
        let x_offset = rasterized.x_offset / scale_factor;
        let y_offset = rasterized.y_offset / scale_factor;
        let advance = rasterized.advance / scale_factor;

        // 登録
        let info = CharInfo {
            alloc_id: allocated.id,
            width,
            height,
            x_offset,
            y_offset,
            advance,
            uv: Vec4::new(x, y, width, height) / CHAR_ATLAS_SIZE as f32,
        };
        self.registereds.push((c, scale), info);

        // 終了
        (self, system)
    }
}

struct RasterizeCharRes {
    data: Vec<u8>,
    width: u32,
    height: u32,
    x_offset: f32,
    y_offset: f32,
    advance: f32,
}

fn rasterize_character(font: &FontRef<'static>, c: char, scale: f32) -> RasterizeCharRes {
    let font = font.as_scaled(scale);
    let glyph = font.scaled_glyph(c);
    let advance = font.h_advance(glyph.id);

    let Some(outlined_glyph) = font.outline_glyph(glyph) else {
        let size = 4 + MARGIN_AXIS;
        return RasterizeCharRes {
            data: vec![0x00; CHAR_ATLAS_CHANNEL_COUNT * size * size],
            width: size as u32,
            height: size as u32,
            x_offset: 0.0,
            y_offset: 0.0,
            advance,
        };
    };

    let width = outlined_glyph.px_bounds().width().ceil() as usize + MARGIN_AXIS;
    let height = outlined_glyph.px_bounds().height().ceil() as usize + MARGIN_AXIS;
    let mut data = vec![0x00; CHAR_ATLAS_CHANNEL_COUNT * width * height];
    outlined_glyph.draw(|x, y, c| {
        let x = x as usize + MARGIN;
        let y = y as usize + MARGIN;
        #[allow(unused_mut)]
        let mut i = CHAR_ATLAS_CHANNEL_COUNT * width * y + CHAR_ATLAS_CHANNEL_COUNT * x;

        // NOTE: macOSでは4チャンネル分更新する。
        #[cfg(target_os = "macos")]
        for _ in 0..3 {
            data[i] = 255;
            i += 1;
        }

        data[i] = (c * 255.0) as u8;
    });

    RasterizeCharRes {
        data,
        width: width as u32,
        height: height as u32,
        x_offset: outlined_glyph.px_bounds().min.x,
        y_offset: outlined_glyph.px_bounds().min.y,
        advance,
    }
}
