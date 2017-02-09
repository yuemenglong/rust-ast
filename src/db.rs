use mysql::Pool;
use mysql::Error;
use mysql::Value;

use mysql::prelude::GenericConnection;
use meta;
use std::fmt::Debug;

// use cond::Cond;
use entity::Entity;
use sql::*;

pub struct DB {
    pub pool: Pool,
}

impl DB {
    pub fn rebuild(&self, meta: &meta::OrmMeta) -> Result<u64, Error> {
        try!(self.drop_tables(meta));
        Ok(try!(self.create_tables(meta)))
    }
    pub fn create_tables(&self, meta: &meta::OrmMeta) -> Result<u64, Error> {
        let mut ret = 0;
        for entity_meta in meta.entities.iter() {
            let sql = sql_create_table(entity_meta);
            println!("{}", sql);
            match self.pool.prep_exec(sql, ()) {
                Ok(res) => ret += res.affected_rows(),
                Err(err) => {
                    return Err(err);
                }
            }
        }
        return Ok(ret);
    }
    pub fn drop_tables(&self, meta: &meta::OrmMeta) -> Result<u64, Error> {
        let mut ret = 0;
        for entity_meta in meta.entities.iter() {
            let sql = sql_drop_table(entity_meta);
            println!("{}", sql);
            match self.pool.prep_exec(sql, ()) {
                Ok(res) => ret += res.affected_rows(),
                Err(err) => {
                    return Err(err);
                }
            }
        }
        return Ok(ret);
    }
    pub fn create_table<E: Entity>(&self) -> Result<u64, Error> {
        let sql = sql_create_table(E::meta());
        println!("{}", sql);
        let res = self.pool.prep_exec(sql, ());
        match res {
            Ok(res) => Ok(res.affected_rows()),
            Err(err) => Err(err),
        }
    }
    pub fn drop_table<E: Entity>(&self) -> Result<u64, Error> {
        let sql = sql_drop_table(E::meta());
        println!("{}", sql);
        let res = self.pool.prep_exec(sql, ());
        match res {
            Ok(res) => Ok(res.affected_rows()),
            Err(err) => Err(err),
        }
    }
    pub fn insert<E: Entity + Clone>(&self, entity: &E) -> Result<E, Error> {
        do_insert(entity, self.pool.get_conn().as_mut().unwrap())
    }
    pub fn update<E: Entity>(&self, entity: &E) -> Result<u64, Error> {
        let sql = sql_update(E::meta());
        println!("{}", sql);
        let mut params = entity.get_params();
        params.push(("id".to_string(), Value::from(entity.get_id())));
        let res = self.pool.prep_exec(sql, params);
        match res {
            Ok(res) => Ok(res.affected_rows()),
            Err(err) => Err(err),
        }
    }
    pub fn get<E: Entity + Default>(&self, id: u64) -> Result<Option<E>, Error> {
        let sql = sql_get(E::meta());
        println!("{}", sql);
        let res = self.pool.prep_exec(sql, vec![("id", id)]);
        if let Err(err) = res {
            return Err(err);
        }
        let mut res = res.unwrap();
        let option = res.next();
        if let None = option {
            return Ok(None);
        }
        let row_res = option.unwrap();
        if let Err(err) = row_res {
            return Err(err);
        }
        let mut row = row_res.unwrap();
        let mut entity = E::default();
        entity.set_values(&res, &mut row, "");
        Ok(Some(entity))
    }
    pub fn delete<E: Entity>(&self, entity: E) -> Result<u64, Error> {
        let sql = sql_delete(E::meta());
        println!("{}", sql);
        let res = self.pool.prep_exec(sql, vec![("id", entity.get_id())]);
        match res {
            Ok(res) => Ok(res.affected_rows()),
            Err(err) => Err(err),
        }
    }
    //     // pub fn select<'a, E: Entity>(&'a self, conds: Vec<Cond>) -> SelectBuilder<'a, E> {
    //     //     SelectBuilder::<'a, E> {
    //     //         pool: &self.pool,
    //     //         conds: RefCell::new(conds),
    //     //         phantom: PhantomData,
    //     //     }
    //     // }
}


fn do_insert<E, C>(entity: &E, conn: &mut C) -> Result<E, Error>
    where E: Entity + Clone,
          C: GenericConnection
{
    // 1. 遍历所有refer，看有没有id，没有的话做insert，有的话暂时什么都不做
    entity.do_insert(conn)
}
