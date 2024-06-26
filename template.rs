//!Write a function string_reverse that takes a &str as input and returns it, reversed as a String ;

fn string_reverse(inp: &str)->String{
    inp.chars().rev().collect()
}

#[runtest]
fn test_inverse(){
    assert_eq!(string_reverse("ciao"), "oaic".to_string());
}


fn bigger(x: i32, y: i32)->i32{
    if x>y{
        x
    }else{
        y
    }
}

#[runtest(2, bigger)]
fn test_bigger(){
    assert_eq!(bigger(5, 3), 5i32);
    assert_eq!(bigger(3, 6), 6i32);
}
#[runtest(2, bigger)]
fn test_bigger_1(){
    assert_eq!(bigger(5, 3), 5i32);
    assert_eq!(bigger(3, 6), 6i32);
}
#[runtest(2, bigger)]
fn test_bigger_2(){
    assert_eq!(bigger(5, 3), 5i32);
    assert_eq!(bigger(3, 6), 6i32);
}
#[runtest(2, bigger)]
fn test_bigger_3(){
    assert_eq!(bigger(5, 3), 5i32);
    assert_eq!(bigger(3, 6), 6i32);
}

/**
. Write a function e_equals_mc_squared that takes as input a f32 representing the mass, and that
uses a globally-defined constant containing the value of the speed of light in a vacuum (expressed in
m/s). The function outputs the energy equivalent to the mass input;
 */
const c: f64 = 300000000.0;
fn e_equals_mc_squared(x: f32)->f64{
    x as f64*c*c
}

#[runtest(2.0)]
fn does_c_exists(){
    //better compile check
    let local_c= c;
}
//does it return f64, and takes f32 as input?



#[runtest(2.0)]
fn does_e_square_work(){
    //better compile check
    let x = 0.1;
    assert_eq!(x as f64*c*c, e_equals_mc_squared(x))
}
