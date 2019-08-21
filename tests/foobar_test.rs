#[test]
fn test_box() {
  let v = 5;
  let b = Box::new(v);
  println!("result: {}",*b+1);
}
