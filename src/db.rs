#[macro_use]
use macros;

use mysql::Pool;
use mysql::Error;
use mysql::PooledConn;

use meta::OrmMeta;
use entity::Entity;
use insert::Insert;
// use session::Session;

pub struct Db {
    pool: Pool,
    orm_meta: &'static OrmMeta,
}

impl Db {
    pub fn new(pool: Pool, orm_meta: &'static OrmMeta) -> Self {
        Db {
            pool: pool,
            orm_meta: orm_meta,
        }
    }
    pub fn rebuild(&self) -> Result<u64, Error> {
        try!(self.drop());
        Ok(try!(self.create()))
    }
    pub fn create(&self) -> Result<u64, Error> {
        let mut ret = 0;
        for entity_meta in self.orm_meta.get_entities().iter() {
            let sql = entity_meta.sql_create_table();
            log!("{}", sql);
            match self.pool.prep_exec(sql, ()) {
                Ok(res) => ret += res.affected_rows(),
                Err(err) => {
                    return Err(err);
                }
            }
        }
        return Ok(ret);
    }
    pub fn drop(&self) -> Result<u64, Error> {
        let mut ret = 0;
        for entity_meta in self.orm_meta.get_entities().iter() {
            let sql = entity_meta.sql_drop_table();
            log!("{}", sql);
            match self.pool.prep_exec(sql, ()) {
                Ok(res) => ret += res.affected_rows(),
                Err(err) => {
                    return Err(err);
                }
            }
        }
        return Ok(ret);
    }
    pub fn get_conn(&self) -> PooledConn {
        self.pool.get_conn().unwrap()
    }
    pub fn insert<E>(&self, entity: &E) -> Result<u64, Error>
        where E: Entity
    {
        let insert = Insert::default::<E>();
        insert.execute(&mut self.get_conn(), entity)
    }
    // fn session_guard<F, R>(&self, f: F) -> R
    //     where F: Fn(&Session) -> R
    // {
    //     let session = self.open_session();
    //     let res = f(&session);
    //     session.close();
    //     res
    // }
    // pub fn open_session(&self) -> Session {
    //     let conn = self.pool.get_conn();
    //     Session::new(conn.unwrap())
    // }
    // pub fn insert<E: Entity>(&self, entity: &E) -> Result<(), Error> {
    //     self.session_guard(|session| session.insert(entity))
    // }
    // pub fn update<E: Entity>(&self, entity: &E) -> Result<(), Error> {
    //     self.session_guard(|session| session.update(entity))
    // }
    // pub fn delete<E: Entity>(&self, entity: &E) -> Result<(), Error> {
    //     self.session_guard(|session| session.delete(entity))
    // }
    // pub fn get<E: Entity>(&self, id: u64) -> Result<Option<E>, Error> {
    //     self.session_guard(|session| session.get::<E>(id))
    // }
}
