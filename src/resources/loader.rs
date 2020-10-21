use {
    crate::{p, new_err, warning, error::GtResult},
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
                    let maxage = get_max_age(&res);
                    let bytes = p!(res.bytes().await).to_vec();
                    if let Err(e) = tokio::task::block_in_place(|| {
                        let lock = self.cache.lock().unwrap();
                        lock.save(url, &bytes, maxage, false)
                    }) { warning!("{}", e) }
                    Ok(bytes)
                } else {
                    Err(new_err!(format!("Failed to load resource {}", url)))
                }
            }
        }

    }

}

fn get_max_age(res: &reqwest::Response) -> u32 {

    const DEFAULT_MAX_AGE: u32 = 300;

    let header_value = match res.headers().get("cache-control") {
        Some(hv) => hv,
        None => return DEFAULT_MAX_AGE
    };

    let header_value_str = match header_value.to_str() {
        Ok(s) => s,
        Err(_) => return DEFAULT_MAX_AGE
    };

    let start_pos = match header_value_str.find("max-age=") {
        Some(p) => p,
        None => return DEFAULT_MAX_AGE
    };

    let max_age = match header_value_str[(start_pos+8)..].parse::<u32>() {
        Ok(ma) => ma,
        Err(_) => return DEFAULT_MAX_AGE
    };

    max_age

}
