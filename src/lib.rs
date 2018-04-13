//! A very simple parser library.

extern crate regex ;

pub use regex::Regex ;

// #[cfg(test)]
mod test ;



/// A position in the parser.
pub struct Pos {
  /// Actual position
  pos: usize,
}


/// Parse result.
pub type PRes<T> = Result<T, ParsErr> ;

/// Parse error.
#[derive(Debug)]
pub struct ParsErr {
  /// Line/col position.
  pos: (usize, usize),
  /// Error messages.
  msg: Vec<String>,
  /// Error line: before error token.
  prf: String,
  /// Error line: error token.
  tkn: String,
  /// Error line: after error token.
  suf: String,
}
impl ParsErr {
  /// Pushes a new error message.
  pub fn push(& mut self, msg: String) {
    self.msg.push(msg)
  }

  /// Position of the error (line, column).
  pub fn pos(& self) -> (usize, usize) {
    self.pos
  }
  /// Error messages.
  pub fn msg(& self) -> & [String] {
    & self.msg
  }
  /// Prefix, token, and suffix of the error line.
  pub fn err(& self) -> (& str, & str, & str) {
    (& self.prf, & self.tkn, & self.suf)
  }

  /// Applies some treatment to each line of the error.
  pub fn default_lines<F: FnMut(& str)>(
    & self, mut treatment: F
  ) {
    treatment(& format!("Error at [{}, {}]", self.pos.0, self.pos.1)) ;
    for msg in & self.msg {
      treatment(msg)
    }
    treatment(& format!("| {}{}{}", self.prf, self.tkn, self.suf)) ;
    treatment(
      &  format!(
        "| {0: ^1$}{2}", "", self.prf.len(),
        & format!("{0:^>1$}", "", self.tkn.len())
      )
    )
  }

  /// Multi-line default representation.
  pub fn default_str(& self) -> String {
    let mut s = String::new() ;
    let mut first = true ;
    self.default_lines(
      |line| {
        if first {
          first = false
        } else {
          s.push('\n')
        }
        s.push_str(line)
      }
    ) ;
    s
  }
}




/// Parser.
pub struct Parser<'s> {
  /// Text being parsed.
  pub text: & 's str,
  /// Current position in the text.
  pos: usize,
  /// Line offset, for errors.
  line_offset: usize,
}

impl<'s> Parser<'s> {
  /// Constructor.
  pub fn new(text: & 's str, line_offset: usize) -> Self {
    Parser { text, pos: 0, line_offset }
  }
  /// Changes the text being parsed.
  ///
  /// Resets the position.
  pub fn set(& mut self, text: & 's str, line_offset: usize) {
    self.text = text ;
    self.line_offset = line_offset
  }

  /// True if at EOF.
  pub fn is_eof(& self) -> bool {
    self.pos >= self.text.len()
  }

  /// The current position.
  pub fn pos(& self) -> Pos {
    Pos { pos: self.pos }
  }

  /// Number of characters left.
  pub fn chars_left(& self) -> usize {
    if self.is_eof() { 0 } else { self.text.len() - self.pos - 1 }
  }

  /// Returns the portion of the text that's not been parsed yet.
  pub fn rest(& self) -> & str {
    & self.text[ self.pos .. ]
  }



  /// Consumes all whitespaces after the current position.
  pub fn ws(& mut self) {
    let rest = & self.text[ self.pos .. ] ;
    let trimmed = rest.trim_left() ;
    let diff = rest.len() - trimmed.len() ;
    self.pos += diff
  }


  /// Tries to parse a tag.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use parsimple::Parser ;
  /// let mut parser = Parser::new("   blah  end", 0) ;
  /// parser.ws() ;
  /// assert_eq! { parser.rest(), "blah  end" }
  /// assert! { parser.try_tag("blah") }
  /// assert_eq! { parser.rest(), "  end" }
  ///
  /// assert! { ! parser.try_tag("end") }
  /// assert_eq! { parser.rest(), "  end" }
  /// ```
  pub fn try_tag(& mut self, tag: & str) -> bool {
    if self.chars_left() < tag.len() {
      false
    } else if & self.text[self.pos .. self.pos + tag.len()] == tag {
      self.pos += tag.len() ;
      true
    } else {
      false
    }
  }
  /// Parses a tag or fails.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use parsimple::Parser ;
  /// let mut parser = Parser::new("   blah  end", 0) ;
  /// parser.ws() ;
  /// assert_eq! { parser.rest(), "blah  end" }
  /// parser.tag("blah").unwrap() ;
  /// assert_eq! { parser.rest(), "  end" }
  ///
  /// let err = parser.tag("end").unwrap_err() ;
  /// assert_eq! {
  ///   err.default_str(),
  ///   "\
  /// Error at [1, 8]
  /// expected tag `end`
  /// |    blah  end
  /// |        ^\
  ///   "
  /// }
  /// ```
  pub fn tag(& mut self, tag: & str) -> PRes<()> {
    if ! self.try_tag(tag) {
      Err(
        self.error_here(
          format!("expected tag `{}`", tag)
        )
      )
    } else {
      Ok(())
    }
  }


  /// Tries to parse a regex.
  ///
  /// A regex's result is only considered relevant if the match starts at the
  /// current position. Hence, for efficiency reasons, all regexes should start
  /// with `^` indicating the start of the string.
  ///
  /// Otherwise, `Regex` will try to match over the rest of the text in its
  /// entirety, but the result will be ignored by the parser (unless it starts
  /// at the current position).
  ///
  /// See the second call to `try_re` in the example below.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use parsimple::{ Parser, Regex } ;
  ///
  /// let mut parser = Parser::new("   blah  end", 0) ;
  /// parser.ws() ;
  /// assert_eq! { parser.rest(), "blah  end" }
  /// let alpha_re = Regex::new(r"[a-zA-Z]+").unwrap() ;
  /// let res = parser.try_re(& alpha_re) ;
  /// assert_eq! { res, Some("blah".into()) }
  ///
  /// assert_eq! { parser.rest(), "  end" }
  ///
  /// let res = parser.try_re(& alpha_re) ;
  /// assert_eq! { res, None }
  /// ```
  pub fn try_re(& mut self, re: & Regex) -> Option<String> {
    if let Some(found_it) = re.find(& self.text[self.pos ..]) {
      println!("start: {}, end: {}", found_it.start(), found_it.end()) ;
      if found_it.start() == 0 {
        let end = self.pos + found_it.end() ;
        println!("pos: {}, end: {}", self.pos, end) ;
        let start = ::std::mem::replace(& mut self.pos, end) ;
        Some( self.text[start .. self.pos].into() )
      } else {
        None
      }
    } else {
      None
    }
  }
  /// Parses a regex or fails.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use parsimple::{ Parser, Regex } ;
  ///
  /// let mut parser = Parser::new("   blah  end", 0) ;
  /// parser.ws() ;
  /// assert_eq! { parser.rest(), "blah  end" }
  /// let alpha_re = Regex::new(r"[a-zA-Z]+").unwrap() ;
  /// let res = parser.re(& alpha_re).unwrap() ;
  /// assert_eq! { res, "blah".to_string() }
  ///
  /// assert_eq! { parser.rest(), "  end" }
  ///
  /// let err = parser.re(& alpha_re).unwrap_err() ;
  /// assert_eq! {
  ///   err.default_str(),
  ///   "\
  /// Error at [1, 8]
  /// no match for regex `[a-zA-Z]+`
  /// |    blah  end
  /// |        ^\
  ///   "
  /// }
  /// ```
  pub fn re(& mut self, re: & Regex) -> PRes<String> {
    if let Some(res) = self.try_re(re) {
      return Ok(res)
    } else {
      Err(
        self.error_here(
          format!("no match for regex `{}`", re.as_str())
        )
      )
    }
  }




  /// Generates a parse error at the current position.
  pub fn error_here<S: Into<String>>(& self, msg: S) -> ParsErr {
    let pos = self.pos() ;
    self.error(pos, msg)
  }

  /// Generates a parse error at the given position.
  pub fn error<S: Into<String>>(
    & self, pos: Pos, msg: S
  ) -> ParsErr {
    let mut pos = pos.pos ;
    let msg = msg.into() ;
    let mut line_count = self.line_offset ;
    let (mut prf, mut tkn, mut suf) = (
      "".to_string(), "<eof>".to_string(), "".to_string()
    ) ;
    for line in self.text.lines() {
      line_count += 1 ;
      if pos < line.len() {
        prf = line[0..pos].to_string() ;
        tkn = line[pos..(pos + 1)].to_string() ;
        suf = line[(pos + 1)..line.len()].to_string() ;
        break
      } else if pos == line.len() {
        prf = line.into() ;
        tkn = "\\n".into() ;
        suf = "".into() ;
        break
      } else {
        pos -= line.len() + 1
      }
    }
    ParsErr {
      pos: (line_count, pos + 1), msg: vec![msg], prf, tkn, suf
    }
  }
}