use {
    crate::{p, new_err, error::GtResult},
    super::cache::AssetCache,
    std::sync::Mutex
};

pub struct ResourceLoader {
    cache: Mutex<AssetCache>
}

impl ResourceLoader {

    pub fn new(cache_path: &str) -> GtResult<Self> {

        Ok(Self {
            cache: Mutex::new(AssetCache::new(cache_path)?)
        })

    }

    pub async fn load(&self, url: &str) -> GtResult<Vec<u8>> {

        let res = tokio::task::block_in_place(|| {
            let lock = self.cache.lock().unwrap();
            lock.load(url)
        });

        match res? {
            Some(asset) => Ok(asset.data),
            None => {
                let res = p!(reqwest::get(url).await);
                if res.status().is_success() {
                    let bytes = p!(res.bytes().await).to_vec();
                    // The result here is not important
                    drop(tokio::task::block_in_place(|| {
                        let lock = self.cache.lock().unwrap();
                        lock.save(url, &bytes, false)
                    }));
                    Ok(bytes)
                } else {
                    Err(new_err!(format!("Failed to load resource {}", url)))
                }
            }
        }

    }

}
