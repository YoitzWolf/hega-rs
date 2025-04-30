use super::generic::*;
use crate::fmt::{decoder::EposDict, generic::*, oscar::*, phqmd::{PHQMDBlock, PHQMDParticle}};

impl Particle for OscarParticle {

    type Decoder = EposDict;

    fn code(&self, dec: &Self::Decoder) -> i32 {
        self.code
    }

    fn momentum_energy(&self, _dct: &EposDict) -> f64 {
        // self.p0
        (self.p.0.powi(2) + self.p.1.powi(2) + self.p.2.powi(2)).sqrt()
    }

    fn momentum(&self, dec: &Self::Decoder) -> &(f64, f64, f64) {
        &self.p
    }

    fn mass_energy(&self, _dct: &EposDict) -> f64 {
        self.mass
    }

    fn e_charge(&self, dct: &EposDict) -> f64 {
        if let Some(pd) = dct.get(&self.code) {
            pd.charge.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values! {:?}", pd); 0. })
        } else {
            if let Some(pd) = dct.get(&-self.code) {
                -pd.charge.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values (anti)! {:?}", pd); 0. })
            } else {
                panic!("Undefined Particle! {:?}", self)
            }
        }
    }

    fn b_charge(&self, dct: &EposDict) -> f64 {
        if let Some(pd) = dct.get(&self.code) {
            [   pd.ifl1.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values! {:?}", pd); 0 }),
                pd.ifl2.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values! {:?}", pd); 0 }),
                pd.ifl3.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values! {:?}", pd); 0 })
            ].iter().map(
                    |&x| { if (x > 0) { 1. } else if (x < 0) {-1.} else {0.} }
                ).sum::<f64>() / 3.0
        } else {
            -if let Some(pd) = dct.get(&-self.code) {
                [   pd.ifl1.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values (anti)! {:?}", pd); 0 }),
                    pd.ifl2.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values (anti)! {:?}", pd); 0 }),
                    pd.ifl3.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values (anti)! {:?}", pd); 0 })
                ].iter().map(
                        |&x| { if (x > 0) { 1. } else if (x < 0) {-1.} else {0.} }
                    ).sum::<f64>() / 3.0
            } else {
                panic!("Undefined Particle!, {:?}", self)
            }
        }
    }

    fn l_charge(&self, dct: &EposDict) -> f64 {
        if dct.is_lepton(&self.code)  {
            1.0
        } else if dct.is_lepton(&-self.code) {
            -1.0
        } else {
            0.0
        }
    }

    fn is_final(&self, dct: &EposDict) -> bool {
        self.state == 0
    }
}

impl HEPEvent for OSCEposBlock {
    type P = OscarParticle;

    fn particles(&self) -> impl Iterator<Item=&OscarParticle> + Clone {
        self.event.iter()
    }
}

impl Particle for PHQMDParticle {

    type Decoder = EposDict;

    fn code(&self, dec: &Self::Decoder) -> i32 {
        self.code
    }

    fn energy(&self, dec: &EposDict) -> f64 {
        // SELF.E is worse!!!!
        (
            self.momentum_energy(dec).powi(2) + self.mass_energy(dec).powi(2)
        ).sqrt()
    }

    fn momentum_energy(&self, _dct: &EposDict) -> f64 {
        //(self.E.powi(2) - self.mass_energy(_dct).powi(2)).sqrt()

        (self.p.0.powi(2) + self.p.1.powi(2) + self.p.2.powi(2)).sqrt()
    }

    fn mass_energy(&self, dct: &EposDict) -> f64 {
        if let Some(x) = dct.get(&self.code) {
            x.mass.unwrap_or_else(
                || { println!("[WARNING]: Using undefined values! {:?}", x); 0. }
            )
        } else if let Some(x) = dct.get(& (-self.code)) {
            x.mass.unwrap_or_else(
                || { println!("[WARNING]: Using undefined values (anti)! {:?}", x); 0. }
            )
        } else {
            panic!("Undefined Particle!, {:?}", self)
        }
    }

    fn e_charge(&self, dct: &EposDict) -> f64 {
        self.charge.into()
    }

    fn b_charge(&self, dct: &EposDict) -> f64 {
        if let Some(pd) = dct.get(&self.code) {
            [   pd.ifl1.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values! {:?}", pd); 0 }),
                pd.ifl2.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values! {:?}", pd); 0 }),
                pd.ifl3.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values! {:?}", pd); 0 })
            ].iter().map(
                    |&x| { if (x > 0) { 1. } else if (x < 0) {-1.} else {0.} }
                ).sum::<f64>() / 3.0
        } else {
            if let Some(pd) = dct.get(&-self.code) {
                -[   pd.ifl1.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values (anti)! {:?}", pd); 0 }),
                     pd.ifl2.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values (anti)! {:?}", pd); 0 }),
                     pd.ifl3.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values (anti)! {:?}", pd); 0 })
                ].iter().map(
                        |&x| { if (x > 0) { 1. } else if (x < 0) {-1.} else {0.} }
                    ).sum::<f64>() / 3.0
            } else {
                panic!("Undefined Particle!, {:?}", self)
            }
        }
    }

    fn l_charge(&self, dct: &EposDict) -> f64 {
        if dct.is_lepton(&self.code)  {
            1.0
        } else if dct.is_lepton(&-self.code) {
            -1.0
        } else {
            0.0
        }
    }

    fn is_final(&self, dct: &EposDict) -> bool {
        true
    }

    fn momentum(&self, dec: &Self::Decoder) -> &(f64, f64, f64) {
        &self.p
    }
}

impl HEPEvent for PHQMDBlock {
    type P = PHQMDParticle;

    fn particles(&self) -> impl Iterator<Item=&PHQMDParticle> + Clone {
        self.event.iter()
    }
}