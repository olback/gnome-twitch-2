use {
    crate::error::GtResult,
    super::cache::{AssetCache, CacheAsset}
};

pub struct ResourceLoader {
    cache: AssetCache
}

impl ResourceLoader {

    pub fn new(cache_path: &str) -> GtResult<Self> {

        Ok(Self {
            cache: AssetCache::new("assets.db")?
        })

    }

    pub fn load(&self, url: &str) -> GtResult<CacheAsset> {

        match self.cache.load(url)? {
            Some(asset) => Ok(asset),
            None => {
                todo!()
            }
        }

    }

}
