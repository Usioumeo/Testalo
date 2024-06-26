//!you should make a function that returns the bigger element

fn bigger(x: i32, y: i32)->i32{
    if x>y{
        x
    }else{
        y
    }
}


#[runtest(2, bigger)]
/// checking bigger function
fn test_bigger(){
    assert_eq!(bigger(5, 3), 5i32);
    assert_eq!(bigger(3, 6), 6i32);
}