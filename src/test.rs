//! Testing stuff.

#[cfg(test)]
use { Parser } ;

#[test]
fn err() {
  let mut parser = Parser::new(
    "   blah stuff ~", 0
  ) ;
  parser.ws() ;
  assert! { parser.try_tag("blah") }
  parser.ws() ;

  let err = parser.error_here("life sux") ;

  println!("{}", err.default_str()) ;

  assert_eq! {
    err.default_str(), "\
Error at [1, 9]
life sux
|    blah stuff ~
|         ^\
    "
  }
}