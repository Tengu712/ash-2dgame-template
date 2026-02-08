use crate::res::Resource;

#[derive(Default)]
pub struct TextureManageState {
    /// 更新すべきテクスチャディスクリプタの情報列
    ///
    /// NOTE: なくても構わないがアロケーションコストを抑えるために。
    pub images: Vec<(Resource, u32)>,
    /// テクスチャディスクリプタの更新状況
    ///
    /// インデックスはディスクリプタのオフセットを表す。
    pub updateds: Vec<Resource>,
    /// そのフレームで出現したイメージの集合
    ///
    /// NOTE: どうせ数種類しかないのでHashSetよりVecの方が高速のはず。
    /// NOTE: なくても構わないがアロケーションコストを抑えるために。
    pub appeareds: Vec<Resource>,
}

impl TextureManageState {
    pub fn clear(mut self) -> Self {
        self.images.clear();
        self.appeareds.clear();
        self
    }

    pub fn update(mut self, image: Resource) -> (Self, u32) {
        self.appeareds.push(image);

        // 更新済のイメージとして存在するなら何もしない
        if let Some(i) = self.updateds.iter().position(|v| v == &image) {
            (self, i as u32)
        }
        // そうでないなら未出現のスロットを上書きする
        else if let Some(i) = self
            .updateds
            .iter()
            .position(|v| !self.appeareds.iter().any(|x| x == v))
        {
            self.images.push((image, i as u32));
            self.updateds[i] = image;
            (self, i as u32)
        }
        // それもできないなら追加するしかない
        else {
            let i = self.updateds.len() as u32;
            self.images.push((image, i));
            self.updateds.push(image);
            (self, i)
        }
    }
}
