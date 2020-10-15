use {
    crate::{p, new_err, error::GtResult},
    std::{path::PathBuf, fs},
    rusqlite::{Connection, params}
};

const TABLE_NAME: &'static str = "assets";
const TABLE_LAYOUT: &'static str = include_str!("../assets.sql");

pub struct Asset {
    pub id: u32,
    pub created: u32,
    pub key: String,
    pub data: Vec<u8>
}

#[derive(Debug)]
pub struct AssetCache {
    path: PathBuf,
    con: Connection
}

impl AssetCache {

    pub fn new(path: &str) -> GtResult<Self> {

        let path = Self::_path()?.join("assets.db");
        let con = p!(Connection::open(&path));
        p!(con.execute(TABLE_LAYOUT, params![]));

        Ok(Self {
            path,
            con
        })

    }

    pub fn load(&self, name: &str) -> GtResult<Option<Asset>> {

        if name.len() < 1 {
            return Err(new_err!("Name may not be less than one byte"))
        }

        todo!()

    }

    pub fn save(&self, name: &str, data: &[u8], overwrite: bool) -> GtResult<()> {

        if name.len() < 1 {
            return Err(new_err!("Name may not be less than one byte"))
        }

        let key = Self::key(name);

        if overwrite {
            let mut stmt = p!(self.con.prepare("delete from assets where key = ?1"));
            p!(stmt.execute(params![key]));
        }

        let mut stmt = p!(self.con.prepare("insert into assets (key, data) VALUES(?1, ?2)"));
        p!(stmt.execute(params![key, data]));

        Ok(())

    }

    pub fn exists(&self, name: &str) -> GtResult<bool> {

        todo!()

    }

    pub fn remove_old(&self, timestamp: u64) -> GtResult<()> {

        todo!()

    }

    pub fn clear(&self) -> GtResult<()> {

        todo!()

    }

    pub fn path(&self) -> PathBuf {

        self.path.clone()

    }

    fn key(name: &str) -> String {

        hex::encode(name)

    }

    fn _path() -> GtResult<PathBuf> {

        let p = p!(dirs::cache_dir().map(|p| p.join("gt2")).ok_or("Failed to obtain cache dir"));

        if !p.exists() {
            p!(fs::create_dir_all(&p));
        }

        Ok(p)

    }

}
