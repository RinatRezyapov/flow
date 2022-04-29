use std::cmp;

use crate::flowenv::*;
use crate::primitives::Primitive;
use crate::dataobject::*;
use crate::dataarray::*;
use crate::data::*;
use crate::command::Command;
use crate::case::*;

#[derive(PartialEq, Debug)]
pub enum CodeException {
    Fail,
    Terminate,
    NextCase,
}

#[derive(Debug)]
pub struct Code {
  pub data: Case,
  pub finishflag: bool,
}

impl Code {
  pub fn new(data: Case) -> Code {
    Code {
      data: data,
      finishflag: false,
    }
  }

  pub fn execute(&mut self, args: DataObject, env:&mut FlowEnv) -> Result<DataObject, CodeException> {
    let mut done = false;
    let mut out = DataObject::new(env);
    
    let mut current_case = self.data.duplicate();
    
    while !done {
      let evaluation: Result<(), CodeException> = (|| {
        let cmds = &mut current_case.cmds;
        let n2 = cmds.len();
        let cons = &mut current_case.cons;
        let n = cons.len();
        
        let mut i = 0;
        while i<n2{
          let cmd = &mut cmds.get_mut(i).unwrap();
          if !cmd.done {
            let mut count = 0;
            let mut b = true;
            let input = &mut cmd.input;
            for (key,value) in input {
              count = count + 1;
              if let Some(_con) = self.lookup_con(cons, &key, "in"){
                b = false;
                break;
              }
              else {
                //println!("No input found!");
                value.done = true;
              }
            }
            if count == 0 || b {
              //println!("NO INPUTS");
              self.evaluate(cmd, env)?;
            }
          }
          i = i + 1;
        }
        
        while !done {
          let mut c = true;
          let mut i = 0;
          while i<n {
            let con = &mut cons[i];
            if !con.done {
              c = false;
              let mut b = false;
              let mut val = Data::DNull;
              let ja = &mut con.src;
              let src = ja.index;
              let srcname = &ja.name;
              let ja = &mut con.dest;
              let dest = ja.index;
              let destname = &ja.name;
              if src == -1 {
                if args.has(&srcname, env){
                  val = args.get_property(&srcname, env);
                }
                b = true;
              }
              else {
                let cmd = &mut cmds[src as usize];
                if cmd.done {
                  val = cmd.result.as_ref().unwrap().get_property(srcname, env).clone();
                  b = true;
                }
              }
              
              if b {
                con.done = true;
                if dest == -2 {
                  out.set_property(&destname, val, env);
                }
                else {
                  let cmd = &mut cmds[dest as usize];
                  if cmd.cmd_type == "undefined" {
                    // FIXME - is this used?
                    println!("Marking undefined command as done");
                    cmd.done = true;
                  }
                  else {
                    let var = &mut cmd.input.get_mut(destname).unwrap();
                    var.val = val;
                    var.done = true;
                    
                    let input = &mut cmd.input;
                    for (_key,v) in input {
                      b = b && v.done;
                      if !b { break; }
                    }
                    if b { 
                      //println!("WITH INPUTS {:?}", cmd.input);
                      self.evaluate(cmd, env)?; 
                    }
                  }
                }
              }
            }
            i = i + 1;
          }
          if c {
            done = true;
          }
        }
        Ok(())
      })();
      
//      println!("execute");
//      env.gc();
      
      if let Err(e) = evaluation {
        if e == CodeException::NextCase {
          current_case = *current_case.nextcase.unwrap();
        }
        else if e == CodeException::Terminate {
          break;
        }
        else {
          return Err(e);
        }
      }
      
    }
    
    Ok(out)
  }
    
  fn lookup_con<'m>(&self, cons: &mut Vec<Connection>, key: &str, which: &str) -> Option<usize> {
    let n = cons.len();
    let mut j = 0;
    while j<n{
      let con = &mut cons.get(j).unwrap();
      let mut bar = &con.src;
      if which == "in" { bar = &con.dest }
      if bar.name == key {
        return Some(j);
      }
      j = j + 1;
    }
    
    None
  }

  fn evaluate(&mut self, cmd: &mut Operation, env:&mut FlowEnv) -> Result<DataObject, CodeException> {
    let mut in1 = DataObject::new(env);
    let in2 = &mut cmd.input;
    //println!("cloning {:?}", in2);
    let mut list_in:Vec<String> = Vec::new();
    for (name,in3) in in2 {
      let dp3 = &mut in3.val;
      in1.set_property(&name, dp3.clone(), env);
      
      if in3.mode == "list" { list_in.push(name.to_string()); }
    }
    
    //let out2 = &cmd.output;
    let mut list_out:Vec<String> = Vec::new();    
    let mut loop_out:Vec<String> = Vec::new();    
    for (name,out3) in &mut cmd.output {
      if out3.mode == "list" { list_out.push(name.to_string()); }
      else if out3.mode == "loop" { loop_out.push(name.to_string()); }    
    }
    
    let n = list_in.len();
    if n == 0 && loop_out.len() == 0 {
      return self.evaluate_operation(cmd, in1, env);
    }
    else {
      let mut out3 = DataObject::new(env);
      for key in &list_out { out3.put_list(&key, DataArray::new(env), env); }
      let mut count = 0;
      if n>0 {
        count = in1.get_array(&list_in[0], env).len(env);
        let mut i = 1;
        while i<n {
          count = cmp::min(count, in1.get_array(&list_in[i], env).len(env));
          i = i + 1;
        }
      }
      
      let mut i = 0;
      loop {
        let mut in3 = DataObject::new(env);
        let list = in1.duplicate(env).keys(env);
        for key in list {
          if !list_in.contains(&key) { 
            let dp = in1.get_property(&key, env);
            in3.set_property(&key, dp, env); 
          }
          else {
            let ja = in1.get_array(&key, env);
            let dp = ja.get_property(i, env);
            in3.set_property(&key, dp, env); 
          }
        }

        let res = self.evaluate_operation(cmd, in3, env)?;
        
        for (k,v) in &mut cmd.output {
          let dp = res.get_property(k, env).clone();
          if list_out.contains(&k) {
            out3.get_array(&k, env).push_property(dp.clone(), env);
          }
          else {
            out3.set_property(&k, dp.clone(), env);
            if loop_out.contains(&k) {
              let newk = &mut v.looop.as_ref().unwrap();
              in1.set_property(&newk, dp.clone(), env);
            }
          }
        }
        
//        println!("evaluate");
//        env.gc();
        
        if cmd.FINISHED {
          break;
        }
        
        if n>0 {
          i = i + 1;
          if i == count {
            break;
          }
        }
      }
      
//      println!("LIST/LOOP END {:?}", out3.to_json(env));
      cmd.result = Some(out3.duplicate(env));
      return Ok(out3);
    }
  }

  fn evaluate_operation(&mut self, cmd:&mut Operation, in1:DataObject, env:&mut FlowEnv) -> Result<DataObject, CodeException> {
    let mut out = DataObject::new(env); // FIXME - Don't instantiate here, leave unassigned
    let cmd_type = &cmd.cmd_type;
    let mut b = true;
    let v = &cmd.name;
    
    let evaluation: Result<(), CodeException> = (|| {
      if cmd_type == "primitive" { // FIXME - use match
        let p = Primitive::new(v);
        out = p.execute(in1, env);
      }
      else if cmd_type == "local" {
//        println!("before local {:?}", in1.to_json(env));
        let src = cmd.localdata.as_ref().unwrap();
        let mut code = Code::new(src.duplicate());
        out = code.execute(in1, env)?;
        cmd.FINISHED = code.finishflag;
      }
      else if cmd_type == "constant" {
        for (key,_x) in &mut cmd.output {
          let ctype = cmd.ctype.as_ref().unwrap();
          if ctype == "int" { out.put_i64(&key, v.parse::<i64>().unwrap(), env); }
          else if ctype == "decimal" { out.put_float(&key, v.parse::<f64>().unwrap(), env); }
          else if ctype == "boolean" { out.put_bool(&key, v.parse::<bool>().unwrap(), env); }
          else if ctype == "string" { out.put_str(&key, v, env); }
          else if ctype == "object" { 
            out.put_object(&key, DataObject::from_json(serde_json::from_str(v).unwrap(), env), env); 
          }
          else if ctype == "array" { 
            out.put_list(&key, DataArray::from_json(serde_json::from_str(v).unwrap(), env), env); 
          }
          else { out.put_null(v, env); }
        }
      }  
      else if cmd_type == "command" {
        let cmdstr = cmd.cmd.as_ref().unwrap();
        let sa = cmdstr.split(":").collect::<Vec<&str>>();
        let lib = sa[0];
        let cmdname = sa[2];
        let mut params = DataObject::new(env);
        for (key,v) in in1.objects(env) {
          params.set_property(&key, v, env);
        }
        
        // FIXME - add remote command support
        // if cmd.has("uuid") {}
        // else {

        let subcmd = Command::new(lib, cmdname, env);
        let result = subcmd.execute(params, env)?;
        
        // FIXME - mapped by order, not by name
        let mut i = 0;
        let cmdout = &mut cmd.output;
        
        
//        let keys = subcmd.src.output.keys().collect::<Vec<_>>().try_into().unwrap();
        let mut keys = Vec::<&str>::new();
        for k in subcmd.src.output.keys() { keys.push(k); }
        
        
        for (key1, _v) in cmdout {
          let key2:&str = &keys[i];
          let dp = result.get_property(key2, env);
          out.set_property(&key1, dp, env);
          i = i + 1;
        }
      }
      else if cmd_type == "match" {
        let key = &in1.duplicate(env).keys(env)[0];
        let ctype = cmd.ctype.as_ref().unwrap();
        let dp1 = &in1.get_property(key, env);
        
        // FIXME - Support match on null?
        if ctype == "int" {
          if !dp1.is_int() { b = false; }
          else {
            let val1 = dp1.int();
            let val2 = v.parse::<i64>().unwrap();
            b = val1 == val2;
          }
        }
        else if ctype == "decimal" {
          if !dp1.is_float() { b = false; }
          else {
            let val1 = dp1.float();
            let val2 = v.parse::<f64>().unwrap();
            b = val1 == val2;
          }
        }
        else if ctype == "boolean" {
          if !dp1.is_boolean() { b = false; }
          else {
            let val1 = dp1.boolean();
            let val2 = v.parse::<bool>().unwrap();
            b = val1 == val2;
          }
        }
        else if ctype == "string" {
          if !dp1.is_string() { b = false; }
          else {
            let val1 = dp1.string();
            b = val1 == v.to_owned(); 
          }
        }
        else {
          // FIXME - Objects & Arrays can't match a constant?
          b = false;
        }
        
      }
      else {
        println!("UNIMPLEMENTED OPERATION TYPE {}", cmd_type);
      }
      Ok(())
    })();
    
//    println!("operation");
    env.gc();
    
    if let Err(e) = evaluation {
      if e == CodeException::Fail {
        b = false;
      }
      else {
        return Err(e);
      }
    }
    
    if cmd_type != "constant" && !cmd.condition.is_none() {
      let condition = &mut cmd.condition.as_ref().unwrap();
      self.evaluate_conditional(&condition.rule, condition.value, b)?;
    }

    cmd.result = Some(out.duplicate(env));
    cmd.done = true;
    
    //println!("OP DONE {} {:?}", cmd_type, out.to_json(env));
    Ok(out)
  }
  
  fn evaluate_conditional(&mut self, c_rule:&str, c_val:bool, m:bool) -> Result<(), CodeException> {
    if c_val == m {
      if c_rule == "next" { return Err(CodeException::NextCase); }
      if c_rule == "terminate" { return Err(CodeException::Terminate); }
      if c_rule == "fail" { return Err(CodeException::Fail); }
      if c_rule == "finish" { self.finishflag = true; }
    }
    
    Ok(())
  }
}

