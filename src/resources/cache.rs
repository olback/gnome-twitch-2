use {
    crate::{p, new_err, error::GtResult},
    std::{path::PathBuf, fs},
    rusqlite::{Connection, params}
};

const TABLE_LAYOUT: &'static str = include_str!("../../assets.sql");

#[derive(Debug)]
pub struct CacheAsset {
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

    pub fn load(&self, name: &str) -> GtResult<Option<CacheAsset>> {

        if name.len() < 1 {
            return Err(new_err!("Name may not be less than one byte"))
        }

        let key = Self::key(name);

        let mut stmt = p!(self.con.prepare("select * from assets where key = ?1"));
        let mut results = p!(stmt.query_map(params![key], |row| {
            Ok(CacheAsset {
                id: row.get("id")?,
                created: row.get("created")?,
                key: row.get("key")?,
                data: row.get("data")?
            })
        })).map(|a| a.unwrap()).collect::<Vec<_>>();

        match results.len() {
            0 => Ok(None),
            1 => Ok(Some(results.remove(0))),
            _ => unreachable!()
        }

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

        let key = Self::key(name);
        let mut stmt = p!(self.con.prepare("select key from assets where key = ?1"));
        let mut res = p!(stmt.query(params![key]));
        let next = p!(res.next());

        Ok(next.is_some())

    }

    // pub fn remove_old(&self, timestamp: u32) -> GtResult<()> { }

    pub fn clear(&self) -> GtResult<()> {

        p!(self.con.execute("delete from assets", params![]));
        Ok(())

    }

    pub fn path(&self) -> &PathBuf {

        &self.path

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
