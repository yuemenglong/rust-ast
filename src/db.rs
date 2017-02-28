use mysql::Pool;
use mysql::Error;
use mysql::Value;

use mysql::prelude::GenericConnection;
use meta;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

// use cond::Cond;
use entity::Entity;
use entity::EntityInner;

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
            let sql = entity_meta.sql_create_table();
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
            let sql = entity_meta.sql_drop_table();
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
        let sql = E::meta().sql_create_table();
        println!("{}", sql);
        let res = self.pool.prep_exec(sql, ());
        match res {
            Ok(res) => Ok(res.affected_rows()),
            Err(err) => Err(err),
        }
    }
    pub fn drop_table<E: Entity>(&self) -> Result<u64, Error> {
        let sql = E::meta().sql_drop_table();
        println!("{}", sql);
        let res = self.pool.prep_exec(sql, ());
        match res {
            Ok(res) => Ok(res.affected_rows()),
            Err(err) => Err(err),
        }
    }
    pub fn insert<E: Entity + Clone>(&self, entity: &E) -> Result<(), Error> {
        let inner_rc = entity.inner();
        let mut inner = inner_rc.borrow_mut();
        // let mut conn = self.pool.get_conn().as_mut().unwrap();
        do_insert(inner.deref_mut(), self.pool.get_conn().as_mut().unwrap())
    }
    pub fn update<E: Entity>(&self, entity: &E) -> Result<u64, Error> {
        let sql = E::meta().sql_update();
        println!("{}", sql);
        let mut params = entity.do_inner(|inner| inner.get_params());
        params.push(("id".to_string(), Value::from(entity.get_id())));
        let res = self.pool.prep_exec(sql, params);
        match res {
            Ok(res) => Ok(res.affected_rows()),
            Err(err) => Err(err),
        }
    }
    pub fn get<E: Entity>(&self, id: u64) -> Result<Option<E>, Error> {
        let sql = E::meta().sql_get();
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
        entity.do_inner_mut(|inner| inner.set_values(&res, &mut row, ""));
        Ok(Some(entity))
    }
    pub fn delete<E: Entity>(&self, entity: E) -> Result<u64, Error> {
        let sql = E::meta().sql_delete();
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


fn do_insert<C>(inner: &mut EntityInner, conn: &mut C) -> Result<(), Error>
    where C: GenericConnection
{
    // 遍历pointer
    for (refer_field, opt) in &inner.pointer_map {
        if opt.is_none() {
            // lazy load, 在insert的情况下基本不会到这个分支
            unreachable!();
        }
        let refer_inner_rc = opt.as_ref().unwrap().clone();
        // 拿到该引用对应的meta信息
        let refer_meta = inner.meta.field_map.get(refer_field).unwrap();
        // 判断是否需要级联写入
        if !refer_meta.has_refer_cascade_insert() {
            continue;
        }
        let refer_id_field = refer_meta.get_pointer_id();
        let mut refer_inner = refer_inner_rc.borrow_mut();
        try!(do_insert(refer_inner.deref_mut(), conn));
        // 将refer的id写回原对象对应的refer_id
        let refer_id = refer_inner.field_map.get("id").unwrap().clone();
        inner.field_map.insert(refer_id_field, refer_id);
    }

    try!(inner.do_insert(conn));

    // 需要等a写入后才能写b，因为aid在b上，需要a先有id
    for (refer_field, opt) in &inner.one_one_map {
        if opt.is_none() {
            // lazy load, 在insert的情况下基本不会到这个分支
            unreachable!();
        }
        let refer_inner_rc = opt.as_ref().unwrap().clone();
        // 拿到该引用对应的meta信息
        let refer_meta = inner.meta.field_map.get(refer_field).unwrap();
        // 判断是否需要级联写入
        if !refer_meta.has_refer_cascade_insert() {
            continue;
        }
        let refer_id_field = refer_meta.get_one_one_id();
        let mut refer_inner = refer_inner_rc.borrow_mut();
        let self_id = inner.field_map.get("id").unwrap().clone();
        refer_inner.field_map.insert(refer_id_field.to_string(), self_id);
        try!(do_insert(refer_inner.deref_mut(), conn));
    }
    Ok(())
}
