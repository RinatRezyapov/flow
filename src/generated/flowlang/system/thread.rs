use ndata::dataobject::*;
use ndata::data::*;
use std::thread;

use crate::command::Command;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("lib");
let a1 = o.get_string("ctl");
let a2 = o.get_string("cmd");
let a3 = o.get_object("params");
let ax = thread(a0, a1, a2, a3);
let mut o = DataObject::new();
o.put_i64("a", ax);
o
}

pub fn thread(mut lib:String, mut ctl:String, mut cmd:String, mut params:DataObject) -> i64 {
thread::spawn(move || {
  let cmd = Command::lookup(&lib, &ctl, &cmd);
  let _x = cmd.execute(params).unwrap();
});
1
}

