use super::generic::*;
use crate::fmt::{generic::*, oscar::*};

impl Particle for OscarParticle {
    fn momentum_energy(&self) -> f64 {
        self.p0
    }

    fn mass_energy(&self) -> f64 {
        self.mass
    }

    fn e_charge(&self) -> f64 {
        todo!()
    }

    fn b_charge(&self) -> f64 {
        todo!()
    }

    fn l_charge(&self) -> f64 {
        todo!()
    }

    fn is_final(&self) -> bool {
        todo!()
    }
}