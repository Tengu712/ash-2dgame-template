use crate::{
    System,
    graphics::{self, CHAR_ATLAS_CHANNEL_COUNT},
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
pub enum CharInfo {
    Rasterizable {
        alloc_id: AllocId,
        width: f32,
        height: f32,
        x_offset: f32,
        y_offset: f32,
        advance: f32,
        uv: Vec4,
    },
    Unrasterizable {
        advance: f32,
    },
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
        let atlas_size = graphics::calc_char_atlas_size(system.window.get_scale_factor());
        Self {
            font: FontRef::try_from_slice(FONT).expect_log("failed to load a font"),
            atlas: AtlasAllocator::new(etagere::size2(atlas_size as i32, atlas_size as i32)),
            registereds: LruCache::unbounded(),
        }
    }
}

impl CharsManageState {
    /// スケールされたフォントのトップ-ベースライン間を取得するメソッド
    ///
    /// WARN: 物理解像度と論理解像度の違いを考慮しない。
    ///       このメソッドの使い道ゆえ。
    pub fn ascent(&self, scale: u32) -> f32 {
        self.font.as_scaled(scale as f32).ascent()
    }

    /// スケールされたフォントの行間を取得するメソッド
    ///
    /// WARN: 物理解像度と論理解像度の違いを考慮しない。
    ///       このメソッドの使い道ゆえ。
    pub fn line_gap(&self, scale: u32) -> f32 {
        self.font.as_scaled(scale as f32).line_gap()
    }

    /// スケールされたフォントのカーニング量を取得するメソッド
    ///
    /// WARN: 物理解像度と論理解像度の違いを考慮しない。
    ///       このメソッドの使い道ゆえ。
    pub fn kern(&self, scale: u32, first: char, second: char) -> f32 {
        let first = self.font.glyph_id(first);
        let second = self.font.glyph_id(second);
        self.font.as_scaled(scale as f32).kern(first, second)
    }

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
        let rasterized = match rasterize_character(&self.font, c, scale as f32 * scale_factor) {
            Ok(n) => n,
            Err(advance) => {
                self.registereds
                    .push((c, scale), CharInfo::Unrasterizable { advance });
                return (self, system);
            }
        };

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
                if let CharInfo::Rasterizable { alloc_id, .. } = least_used.1 {
                    self.atlas.deallocate(alloc_id);
                }
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

        // 登録
        let width = rasterized.width as f32 - MARGIN_AXIS as f32;
        let height = rasterized.height as f32 - MARGIN_AXIS as f32;
        let atlas_size = graphics::calc_char_atlas_size(scale_factor);
        let info = CharInfo::Rasterizable {
            alloc_id: allocated.id,
            width: width / scale_factor,
            height: height / scale_factor,
            x_offset: rasterized.x_offset / scale_factor,
            y_offset: rasterized.y_offset / scale_factor,
            advance: rasterized.advance / scale_factor,
            uv: Vec4::new(
                allocated.rectangle.min.x as f32 + MARGIN as f32,
                allocated.rectangle.min.y as f32 + MARGIN as f32,
                width,
                height,
            ) / atlas_size as f32,
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

/// 文字をラスタライズする関数
///
/// ラスタライザブルでない文字に対してはadvanceと共に`Err`を返す。
///
/// 物理解像度と論理解像度の違いを考慮しない。
/// 考慮したいなら`scale`を大きくすると良い。
fn rasterize_character(
    font: &FontRef<'static>,
    c: char,
    scale: f32,
) -> Result<RasterizeCharRes, f32> {
    let font = font.as_scaled(scale);
    let glyph = font.scaled_glyph(c);
    let advance = font.h_advance(glyph.id);

    let Some(outlined_glyph) = font.outline_glyph(glyph) else {
        return Err(advance);
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

    Ok(RasterizeCharRes {
        data,
        width: width as u32,
        height: height as u32,
        x_offset: outlined_glyph.px_bounds().min.x,
        y_offset: outlined_glyph.px_bounds().min.y,
        advance,
    })
}
