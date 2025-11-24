use super::generic::*;
use crate::fmt::{decoder::{EposDict, EposDictParticle}, generic::*, oscar::*, phqmd::{PHQMDBlock, PHQMDParticle}, qgsm::{QGSMBlock, QGSMParticle}};

impl Particle for EposDictParticle {
    type Decoder = ();

    fn momentum_energy(&self, dec: &Self::Decoder) -> f64 {
        0.0
    }

    fn momentum(&self, dec: &Self::Decoder) -> &(f64, f64, f64) {
        &(0.0, 0.0, 0.0)
    }

    fn mass_energy(&self, dec: &Self::Decoder) -> f64 {
        self.mass.unwrap()
    }

    fn e_charge(&self, dec: &Self::Decoder) -> f64 {
        self.charge.unwrap()
    }

    fn b_charge(&self, dec: &Self::Decoder) -> f64 {
        //                                             1000010020
        // println!(">{} : {}\n", self.id_PDG.unwrap(), self.id_PDG.unwrap().abs() - 1000000000);
        if (self.id_PDG.unwrap().abs() > 1000000000) {
            // nuclei
            let code = self.id_PDG.unwrap();
            let baryon = (code.abs() % 10000) / 10;
            (baryon as f64) * (code.signum() as f64)
        } else {
            if self.id_PDG.unwrap() > 0 {
                [   self.ifl1.unwrap_or_else(|| { println!("[WARNING::EPOS DICT]: Using undefined values! {:?}", self); 0 }),
                    self.ifl2.unwrap_or_else(|| { println!("[WARNING::EPOS DICT]: Using undefined values! {:?}", self); 0 }),
                    self.ifl3.unwrap_or_else(|| { println!("[WARNING::EPOS DICT]: Using undefined values! {:?}", self); 0 })
                ].iter().map(
                        |&x| { if (x > 0) { 1. } else if (x < 0) {-1.} else {0.} }
                    ).sum::<f64>() / 3.0
            } else {
                -[  self.ifl1.unwrap_or_else(|| { println!("[WARNING::EPOS DICT]: Using undefined values (anti)! {:?}", self); 0 }),
                    self.ifl2.unwrap_or_else(|| { println!("[WARNING::EPOS DICT]: Using undefined values (anti)! {:?}", self); 0 }),
                    self.ifl3.unwrap_or_else(|| { println!("[WARNING::EPOS DICT]: Using undefined values (anti)! {:?}", self); 0 })
                ].iter().map(
                        |&x| { if (x > 0) { 1. } else if (x < 0) {-1.} else {0.} }
                    ).sum::<f64>() / 3.0
            }
        }
    }

    fn l_charge(&self, dec: &Self::Decoder) -> f64 {
        self.lepton_charge
    }

    fn is_final(&self, dec: &Self::Decoder) -> bool {
        true
    }

    fn code(&self, dec: &Self::Decoder) -> i32 {
        self.id_PDG.unwrap()
    }
}

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
            /*[   pd.ifl1.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values! {:?}", pd); 0 }),
                pd.ifl2.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values! {:?}", pd); 0 }),
                pd.ifl3.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values! {:?}", pd); 0 })
            ].iter().map(
                    |&x| { if (x > 0) { 1. } else if (x < 0) {-1.} else {0.} }
                ).sum::<f64>() / 3.0*/
            pd.b_charge(&())
        } else {
            if let Some(pd) = dct.get(&-self.code) {
                /*-[   pd.ifl1.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values (anti)! {:?}", pd); 0 }),
                    pd.ifl2.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values (anti)! {:?}", pd); 0 }),
                    pd.ifl3.unwrap_or_else(|| { println!("[WARNING::EPOS]: Using undefined values (anti)! {:?}", pd); 0 })
                ].iter().map(
                        |&x| { if (x > 0) { 1. } else if (x < 0) {-1.} else {0.} }
                    ).sum::<f64>() / 3.0*/
                -pd.b_charge(&())
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

    /*fn energy(&self, dec: &EposDict) -> f64 {
        // SELF.E is worse!!!!
        (
            self.momentum_energy(dec).powi(2) + self.mass_energy(dec).powi(2)
        ).sqrt()
    }*/

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
                pd.b_charge(&())
        } else {
           if let Some(pd) = dct.get(&-self.code) {
                -pd.b_charge(&())
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


impl Particle for QGSMParticle {
    type Decoder = EposDict;

    fn momentum_energy(&self, dec: &Self::Decoder) -> f64 {
        (self.p.0.powi(2) + self.p.1.powi(2) + self.p.2.powi(2)).sqrt()
    }

    fn momentum(&self, dec: &Self::Decoder) -> &(f64, f64, f64) {
        &self.p
    }

    fn mass_energy(&self, dec: &Self::Decoder) -> f64 {
        self.mass
    }

    fn e_charge(&self, dec: &Self::Decoder) -> f64 {
        self.charge as f64
    }

    fn b_charge(&self, dec: &Self::Decoder) -> f64 {
        self.baryon_number as f64
    }

    fn l_charge(&self, dec: &Self::Decoder) -> f64 {
        self.lepton_number as f64
    }

    fn is_final(&self, dec: &Self::Decoder) -> bool {
        true
    }

    fn code(&self, dec: &Self::Decoder) -> i32 {
        self.code
    }
}

impl HEPEvent for QGSMBlock {
    type P = QGSMParticle;
    fn particles(&self) -> impl Iterator<Item=&QGSMParticle> + Clone {
        self.event.iter()
    }
}