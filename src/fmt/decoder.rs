use std::{collections::{HashMap, HashSet}, io::BufRead, str::FromStr};

use serde::{Deserialize, Serialize};


/// column 1 : id_EPOS     : Option<i32>
/// column 2 : id_PDG      : Option<i32>                                                                                         
/// column 3 : id_QGSJET   : Option<i32>                                                                                    
/// column 4 : id_GHEISHA  : Option<i32>                                                                                     
/// column 5 : id_SIBYLL   : Option<i32>                                                                                
/// column 6 : Name of the particle  : String                                                                          
/// column 7 : ifl1  : Option<i32> !Quark flavors:                                                                            
/// colunm 8 : ifl2  : Option<i32> !Baryons: ifl1,ifl2,ifl3                                                                   
/// column 9 : ifl3  : Option<i32> !Mesons: ifl2,ifl3  Quarks: ifl3  Diquarks ifl1,ifl2                                       
/// columnn 10 : Counter : Option<i32> ! (formely jspin variable)                                                               
/// column 11 : Mass : Option<f64> (in units of GeV/c^2)                                                                      
/// column 12 : Charge : Option<f64> (in units of e)            
/// column 13 : Width : Option<f64> (in units of GeV)                                                              
/// column 14 : Multiplicity variable : 1 wrote particle+anti, 2 : wrote just the particle
/// column 15 : Degeneracy (2*J + 1), (eg=educated guess refer to unknown J in PDG)
/// column 16 : Status (R = well established,      A = well established with guessed degeneracy,
///                     D = established,           B = established with guessed degeneracy,
///                     S = not well established,  T=  not well established with guessed degeneracy,
///                     F = poorly established, 
///                     M = not in PDG list,       N= not in PDG list with guessed degeneracy)
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EposDictParticle{
    pub id_EPOS     : Option<i32>,
    pub id_PDG      : Option<i32>,
    pub id_QGSJET   : Option<i32>,
    pub id_GHEISHA  : Option<i32>,
    pub id_SIBYLL   : Option<i32>,
    pub name        : String,
    pub ifl1        : Option<i32>,
    pub ifl2        : Option<i32>,
    pub ifl3        : Option<i32>,
    pub counter     : Option<i32>,
    pub mass        : Option<f64>,
    pub charge      : Option<f64>,
    pub width       : Option<f64>,
    pub multiplicity: Option<i32>,
    pub degeneracy  : Option<i32>,
    pub status      : String
}

impl EposDictParticle {
    pub fn new(id_EPOS: Option<i32>, id_PDG: Option<i32>, id_QGSJET: Option<i32>, id_GHEISHA: Option<i32>, id_SIBYLL: Option<i32>, name: String, ifl1: Option<i32>, ifl2: Option<i32>, ifl3: Option<i32>, counter: Option<i32>, mass: Option<f64>, charge: Option<f64>, width: Option<f64>, multiplicity: Option<i32>, degeneracy: Option<i32>, status: String) -> Self {
        Self { id_EPOS, id_PDG, id_QGSJET, id_GHEISHA, id_SIBYLL, name, ifl1, ifl2, ifl3, counter, mass, charge, width, multiplicity, degeneracy, status }
    }

    fn cleared<'a, T: FromStr>(c: &'a str) -> Option<T>
    where <T as FromStr>::Err: std::fmt::Debug {
        if c.eq("99") {
            None
        } else {
            // println!(">> \"{}\"", c);
            Some(c.parse::<T>().unwrap())
        }
    }

    pub fn from_str(s: String) -> Self {
        let mut tokens = s.split_ascii_whitespace().filter(|x| {x.trim().len() > 0});
        Self {
            id_EPOS: Self::cleared(tokens.next().unwrap()),
            id_PDG: Self::cleared(tokens.next().unwrap()),
            id_QGSJET: Self::cleared(tokens.next().unwrap()),
            id_GHEISHA: Self::cleared(tokens.next().unwrap()),
            id_SIBYLL: Self::cleared(tokens.next().unwrap()),
            name: tokens.next().unwrap().to_string(),
            ifl1: Self::cleared(tokens.next().unwrap()),
            ifl2: Self::cleared(tokens.next().unwrap()),
            ifl3: Self::cleared(tokens.next().unwrap()),
            counter: Self::cleared(tokens.next().unwrap()),
            mass: Self::cleared(tokens.next().unwrap()),
            charge: Self::cleared(tokens.next().unwrap()),
            width: Self::cleared(tokens.next().unwrap()),
            multiplicity: Self::cleared(tokens.next().unwrap()),
            degeneracy: Self::cleared(tokens.next().unwrap()),
            status: tokens.next().unwrap().to_string()
        }
    }
}

pub enum DctCoding {
    EPOS,
    PDG,
    
}

#[derive(Debug)]
pub struct EposDict {
    dct: HashMap<i32, EposDictParticle>,
    leptons: HashSet<i32>
}

impl EposDict {
    pub fn upload<T: Sized + std::io::Read>(data: std::io::BufReader<T>, as_code: DctCoding, leptons: Option<HashSet<i32>>) -> Self {

        let mut mp = HashMap::new();

        data.lines().for_each(
            |s| {
                if let Ok(s) = s {
                    let s = s.trim();
                    if s.starts_with("!") || s.len() < 10 { /* skip */}
                    else {
                        let v = EposDictParticle::from_str(s.to_string());
                        let code = match as_code {
                            DctCoding::EPOS => v.id_EPOS.unwrap_or(99),
                            DctCoding::PDG => v.id_PDG.unwrap_or(99),
                        };
                        mp.insert(code, v);
                    }
                } else {
                    panic!("ERROR READING DICT WITH EPOS INTERPRETER")
                }
            }
        );
        Self { dct: mp, leptons: leptons.unwrap_or(HashSet::new())}
    }

    pub fn get(&self, k: &i32) -> Option<&EposDictParticle> {
        self.dct.get(k)
    }

    pub fn is_lepton(&self, k: &i32) -> bool {
        self.leptons.contains(k)
    }

    pub fn codes(&self) -> std::collections::hash_map::Keys<'_, i32, EposDictParticle> {
        self.dct.keys()
    }

}