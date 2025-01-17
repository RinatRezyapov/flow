let listener = TcpListener::bind(socket_address).unwrap();
for stream in listener.incoming() {
  let cmd_path = [library.to_owned(), control.to_owned(), command.to_owned()];
  let mut stream = stream.unwrap();
  thread::spawn(move || {
    let remote_addr = stream.peer_addr().unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    let mut count = reader.read_line(&mut line).unwrap();
    if count > 2 {
      line = (&line[0..count-2]).to_string();
      count = line.find(" ").unwrap();
      let method = (&line[0..count]).to_string();
      line = (&line[count+1..]).to_string();
      count = line.find(" ").unwrap();
      let protocol = (&line[count+1..]).to_string();
      let path = (&line[0..count]).to_string();

      let mut headers = DataObject::new();
      let mut last = "".to_string();
      loop {
        let mut line = String::new();
        count = reader.read_line(&mut line).unwrap();
        if count == 2 {
          break;
        }
        if (&line[0..1]).to_string() != " ".to_string(){
          count = line.find(":").unwrap();
          let mut key = (&line[0..count]).to_string();
          key = key.to_uppercase();
          let mut val = (&line[count+1..]).to_string();
          val = val.trim().to_string();
          if !headers.has(&key) {
            headers.put_str(&key, &val);
          }
          else {
            let d = headers.get_property(&key);
            if d.is_array() {
              d.array().push_str(&val);
            }
            else {
              let old = d.string();
              let mut v = DataArray::new();
              v.push_str(&old);
              v.push_str(&val);
              headers.put_list(&key, v);
            }
          }
          last = key;
        }
        else {
          let d = headers.get_property(&last);
          if d.is_array(){
            let mut v = d.array();
            let n = v.len() - 1;
            let mut old = v.get_string(n);
            v.remove_property(n);
            old = old + "\r\n" + line.trim_end();
            v.push_str(&old);
          }
          else {
            let mut old = d.string();
            old = old + "\r\n" + line.trim_end();
            headers.put_str(&last, &old);
          }
        }
      }

      let mut querystring = "".to_string();
      let mut params = DataObject::new();

      if method == "POST" {
        // extractPOSTParams
        let clstr = headers.get_string("CONTENT-LENGTH");
        let ctstr = headers.get_string("CONTENT-TYPE");
        let mut max = clstr.parse::<i64>().unwrap();

        let s = ctstr.to_lowercase();
        if s.starts_with("multipart/") {
          // MULTIPART

          panic!("No MIME MULTIPART support yet");


        }
        else {
          while max > 0 {
            let mut buf = vec![];
            let n = reader.read_until(b'=', &mut buf).expect("reading from cursor won't fail"); // FIXME - WUT?
            max -= n as i64;
            let mut key = std::str::from_utf8(&buf).unwrap().to_string();
            if key.ends_with("=") {
              key = (&key[..n-1]).to_string();
            }

            buf = vec![];
            let n = reader.read_until(b'&', &mut buf).expect("reading from cursor won't fail"); // FIXME - WUT?
            max -= n as i64;
            let mut value = std::str::from_utf8(&buf).unwrap().to_string();
            if value.ends_with("&") {
              value = (&value[..n-1]).to_string();
            }

            key = key.replace("+"," ");
            value = value.replace("+"," ");
            key = hex_decode(key);
            value = hex_decode(value);

            params.put_str(&key, &value);
          }
        }
      }

      let cmd:String;
      if path.contains("?"){
        let i = path.find("?").unwrap();
        cmd = path[0..i].to_string();
        querystring = path[i+1..].to_string();
        let mut oneline = querystring.to_owned();
        let mut oneparam:String;
        while oneline.len() > 0 {
          if oneline.contains("&")  {
            let i = oneline.find("&").unwrap();
            oneparam = oneline[0..i].to_string();
            oneline = oneline[i+1..].to_string();
          }
          else {
            oneparam = oneline;
            oneline = "".to_string();
          }

          if oneparam.contains("=") {
            let i = oneparam.find("=").unwrap();
            let key = oneparam[0..i].to_string();
            let value = oneparam[i+1..].to_string();
            params.put_str(&key, &value);
          }
        }
      }
      else {
        cmd = path;
      }
  /*    
      let mut sid = "".to_string();
      if params.has("sessionid") { sid = params.get_string("sessionid"); }
      else if headers.has("COOKIE"){ 
        let c = headers.get_string("COOKIE");
        let split = c.split("; ");
        for s in split {
          if s.starts_with("sessionid=") {
            sid = s[10..].to_string();
            break;
          }
        }
      }

      if sid == "" { sid = unique_session_id(); }
      headers.put_str("nn-sessionid", &sid);
      params.put_str("sessionid", &sid);

      let mut globals = DataStore::globals();
      if !globals.has("SESSIONS") { globals.put_object("SESSIONS", DataObject::new()); }
      if !globals.has("SESSION_TIMEOUT") { globals.put_i64("SESSION_TIMEOUT", 900000); }
      let mut sessions = globals.get_object("SESSIONS");
      let session_timeout = globals.get_i64("SESSION_TIMEOUT");
      if !sessions.has(&sid) {
        let mut ses = DataObject::new();
        ses.put_i64("expire", time()+session_timeout);
        sessions.put_object(&sid, ses);
      }

      let mut ses = sessions.get_object(&sid);
  */    
      let loc = remote_addr.to_string();
  //    ses.put_str("userlocation", &loc);
      headers.put_str("nn-userlocation", &loc);
  //    params.put_str("userlocation", &loc);

      // FIXME
  //		o = new JSONObject(headers);
  //		params.put("request_headers", o.toString());
  //		params.put("request_input_stream", parser.is);
  //		params.put("request_output_stream", parser.os);

  /*    
      if ses.has("user") {
        headers.put_str("nn-username", &ses.get_string("username"));
        let user = ses.get_object("user");
        let groups:String;
        if user.has("groups") { groups = user.get_string("groups"); }
        else { groups = "anonymous".to_string(); }
        headers.put_str("nn-groups", &groups);
      }
      else {
        headers.put_str("nn-username", "anonymous");
        headers.put_str("nn-groups", "anonymous");
      }
  */    
      let mut request = DataObject::new();
  //    request.put_str("sessionid", &sid);

      // FIXME - Is this necessary?
      if headers.has("ACCEPT-LANGUAGE"){ 
        let lang = headers.get_string("ACCEPT-LANGUAGE");
        request.put_str("language", &lang);
      }
      else {
        request.put_str("language", "*");
      }

      // FIXME - Is this necessary?
      if headers.has("HOST"){ 
        let h = headers.get_string("HOST");
        request.put_str("host", &h);
      }

      // FIXME - Is this necessary?
      if headers.has("REFERER"){ 
        let h = headers.get_string("REFERER");
        request.put_str("referer", &h);
      }

      request.put_str("protocol", &protocol);
      request.put_str("path", &cmd);
      request.put_str("loc", &loc);
      request.put_str("method", &method);
      request.put_str("querystring", &querystring);
      request.put_object("headers", headers.duplicate());
      request.put_object("params", params);
      request.put_i64("timestamp", time());

      // FIXME
  //		CONTAINER.getDefault().fireEvent("HTTP_BEGIN", log);

      // FIXME - implement or remove
      if headers.has("TRANSFER-ENCODING"){
        let trenc = headers.get_string("TRANSFER-ENCODING");
        if trenc.to_uppercase() == "CHUNKED" {
          // CHUNKED
        }
      }

      if headers.has("SEC-WEBSOCKET-KEY") {
        //let key = headers.get_string("SEC-WEBSOCKET-KEY");

        // FIXME - Implement

        panic!("WEBSOCKET NOT IMPLEMENTED");
      }
      else {
        // FIXME - Implement keep-alive
        let mut ka = "close".to_string();
        if headers.has("CONNECTION") { ka = headers.get_string("CONNECTION"); }

        // FIXME - origin is never used
        let mut origin = "null".to_string();
        if headers.has("ORIGIN") { origin = headers.get_string("ORIGIN"); }

        // FIXME
  //			setRequestParameters(params);

        let command = Command::lookup(&cmd_path[0], &cmd_path[1], &cmd_path[2]);
        let mut response = DataObject::new();
        let dataref = response.data_ref;

        let result = panic::catch_unwind(|| {
          let mut p = DataObject::get(dataref);
          let o = command.execute(request).unwrap();
  //        let o = o.get_object("a").duplicate(); // FIXME - Side effect of rust. Require a flow command instead?
          p.put_object("a", o);
        });
        

		match result {
          Ok(_x) => (),
          Err(e) => {
            
            let s = match e.downcast::<String>() {
              Ok(panic_msg) => format!("{}", panic_msg),
              Err(_) => "unknown error".to_string()
            };        
            
            let mut o = DataObject::new();
            let s = format!("<html><head><title>500 - Server Error</title></head><body><h2>500</h2>Server Error: {}</body></html>", s);
            o.put_str("body", &s);
            o.put_i64("code", 500);
            o.put_str("mimetype", "text/html");
            response.put_object("a", o);
          }
		}

/*        
        if result.is_err() {
          let mut o = DataObject::new();
          let s = format!("<html><head><title>500 - Server Error</title></head><body><h2>500</h2>Server Error: {:?}</body></html>", result.unwrap_err());
          o.put_str("body", &s);
          o.put_i64("code", 500);
          o.put_str("mimetype", "text/html");
          response.put_object("a", o);
        }
*/
        let response = response.get_object("a").duplicate();

        let body:String;
        let mimetype:String;
        let len:i64;
        let code:u16;
        let msg:String;
        let mut headers:DataObject;
        
        let isfile = response.has("file") && response.get_property("file").is_string();
        
        if isfile { body = response.get_string("file"); }
        else if response.has("body") && response.get_property("body").is_string() { body = response.get_string("body"); }
        else { body = "".to_owned(); }

        if response.has("code") && response.get_property("code").is_int() { code = response.get_i64("code") as u16; }
        else { code = 200; }

        if response.has("msg") && response.get_property("msg").is_string() { msg = response.get_string("msg"); }
        else { 
          if code < 200 { msg = "INFO".to_string(); }
          else if code < 300 { msg = "OK".to_string(); }
          else if code < 400 { msg = "REDIRECT".to_string(); }
          else if code < 500 { msg = "CLIENT ERROR".to_string(); }
          else { msg = "SERVER ERROR".to_string(); }
        }

        if response.has("headers") && response.get_property("headers").is_object() { headers = response.get_object("headers"); }
        else { headers = DataObject::new(); }

        if response.has("mimetype") && response.get_property("mimetype").is_string() { mimetype = response.get_string("mimetype"); }
        else if headers.has("Content-Type") { mimetype = headers.get_string("Content-Type"); }
        else if isfile { mimetype = mime_type(cmd); }
        else { mimetype = "text/plain".to_string(); }

        if response.has("len") && response.get_property("len").is_int() { len = response.get_i64("len"); }
        else if headers.has("Content-Length") { len = headers.get_i64("Content-Length"); }
        else if isfile { len = fs::metadata(&body).unwrap().len() as i64; }
        else { len = body.len() as i64; }

        //FIXME
  //		int[] range = extractRange(len, h);
  //		if (range[1] != -1) len = range[1] - range[0] + 1;
  //		String res = range[0] == -1 ? "200 OK" : "206 Partial Content";

        let now = Utc::now();
        let date = now.to_rfc2822();

        headers.put_str("Date", &date);
        headers.put_str("Content-Type", &mimetype);
        if len != -1 { headers.put_str("Content-Length", &len.to_string()); }
        // FIXME
  //      if (acceptRanges != null) h.put("Accept-Ranges", acceptRanges);
  //      if (range != null && range[0] != -1) h.put("Content-Range","bytes "+range[0]+"-"+range[1]+"/"+range[2]);
  //      if (expires != -1) h.put("Expires", toHTTPDate(new Date(expires)));

  //      let later = now.add(Duration::weeks(52));
  //      let cookie = "sessionid=".to_string()+&sid+"; Path=/; Expires="+&later.to_rfc2822();
  //      headers.put_str("Set-Cookie", &cookie);

        // FIXME
  //		if (origin != null)
  //		{
  //			String cors = getCORS(name, origin);
  //			if (cors != null)
  //			{
  //				h.put("Access-Control-Allow-Origin", cors);
  //				if (!cors.equals("*")) h.put("Vary", "Origin");
  //			}
  //		}

        let mut reshead = "HTTP/1.1 ".to_string()+&code.to_string()+" "+&msg+"\r\n";
        for (k,v) in headers.objects() {
          reshead = reshead +&k + ": "+&Data::as_string(v)+"\r\n";
        }
        reshead = reshead + "\r\n";
        
        if isfile {
          stream.write(reshead.as_bytes()).unwrap();
          let mut file = fs::File::open(&body).unwrap();
          let chunk_size = 0x4000;
          loop {
            let mut chunk = Vec::with_capacity(chunk_size);
            let n = std::io::Read::by_ref(&mut file).take(chunk_size as u64).read_to_end(&mut chunk).unwrap();
            if n == 0 { break; }
            stream.write(&chunk).unwrap();
            if n < chunk_size { break; }
          }
        }
        else {
          let response = reshead + &body;
          //println!("{}\r\n", &response);
          stream.write(response.as_bytes()).unwrap();
        }
        stream.flush().unwrap();

        // FIXME
  //				clearRequestParameters();

      }
    }
    // FIXME
//				CONTAINER.getDefault().fireEvent("HTTP_END", log);

    DataStore::gc();
  });
}
"OK".to_string()
