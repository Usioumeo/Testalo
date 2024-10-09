use syn::{parse_str, File};

use crate::generatorv2::parser::TestDefinition;

use super::error::RustError;
use super::parser::RustExercise;

#[derive(Clone, Default)]
pub struct GeneratedFiles {}
impl GeneratedFiles {
    pub fn generated(def: RustExercise, user: String) -> Result<Self, RustError> {
        let user: File = parse_str(&user)?;


        let tests: Vec<TestDefinition> = def
            .tests
            .into_iter()
            .map(|x| TestDefinition::try_from(x))
            .collect::<Result<Vec<TestDefinition>, RustError>>()?;
        for i in tests{
            
        }
        //let def = TestDefinition::try_from(def)?;

        todo!()
    }
}
