#![procedural::magic_macro]
//! General problem description
#![dependency("oorandom=11.0")]

fn bigger(i: f32)->f32{hidden::product(i)}

mod hidden{
    pub fn product(i: f32)->f32{i*2.0}
}

#[runtest(1.0)]
#[overwrite(impl product in hidden )]
/// test description
fn test_bigger(){
    let mut rng = oorandom::Rand32::new(0);

    let number = rng.rand_float();
    assert!(number*2.0-bigger(number)<0.0000001);
}