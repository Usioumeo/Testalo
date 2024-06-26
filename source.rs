fn string_reverse(inp: &str)->String{
    inp.chars().rev().collect()
}

fn bigger(x: i32, y: i32)->i32{
    if x>y{
        x
    }else{
        y
    }
}
const c: f64 = 300000000.0;
fn e_equals_mc_squared(x: f32)->f64{
    x as f64*c*c
}