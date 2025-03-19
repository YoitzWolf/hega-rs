
use std::fmt::format;

use crate::anlz::{ScalarCriteria, Particle};

/// Custom paramenter example
/// Can use a Struct instead of Enum
#[derive(Clone, Debug, PartialEq)]
pub enum MyExampleCriterias {
    StupidCriteria1,
    StupidCriteria2(i32) // if we need parameter
}

impl<'a, S, T> ScalarCriteria<'a, S, T> for MyExampleCriterias 
where T: Particle<Decoder = S> + 'static, {

    // Here 
    // p: &T is argument 'p' of type: reference to T where T implements Particle trait
    // from anlz::generic
    // dec is DECODER described im Particle implementation
    // for example, EposDict for EPOS UrQMD or PHQMD
    // 
    fn get_criteria_value(&self, p: &T, dec: &S) -> f64 {
        match self {
            MyExampleCriterias::StupidCriteria1 => {
                // calculate something using p: Particle data
                return 0.0; 
                // or just 0.0 without return and semicolon)))
            },
            MyExampleCriterias::StupidCriteria2(p) => { // p: &i32 <- it is ummutable i32 ref - parameter is defined when criteria object is created
                (*p).into() // just make f64 (64 float) from i32 (ref to 32bit integer)
            },
        }
    }

    fn name(&self) -> String {
        match self {
            MyExampleCriterias::StupidCriteria1 => "Stupid Criteria Name".to_owned(),
            MyExampleCriterias::StupidCriteria2(p) => format!("Stupid Critera 2 with param {}", p),
        }
    }    
}