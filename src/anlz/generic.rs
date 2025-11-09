use rayon::prelude::*;
use std::{collections::HashSet, f64::consts::PI, fmt::Debug, sync::Arc};
use super::{fncs::*, impls};

use crate::fmt::{oscar::{OSCEposBlock, OSCEposDataFile}, phqmd::PHQMDDataFile};

pub trait ScalarCriteria<'a, S, T>: Send + Sync//: PartialEq + Debug + Clone + Send
where T: Particle<Decoder = S> + 'static,
    // S: 'a
{
    fn get_criteria_value(&self, p: &T, dec: &S) -> f64;// Clone + Send + Sync + 'a;

    fn name(&self) -> String;
}

#[derive(Debug, PartialEq, Clone)]
pub enum StandardCriteria {
    FinEnergy,
    ECharge,
    BCharge,
    LCharge,
    FinCnt,
    FinChargedCnt,
    PseudorapidityFilterCnt(f64, f64),
}


impl<'a, S, T> ScalarCriteria<'a, S, T> for StandardCriteria
where T: Particle<Decoder = S> + 'static, {
    fn get_criteria_value(&self, p: &T, dec: &S) -> f64 {//+ Clone + Send + 'a{
        match self {
            StandardCriteria::FinEnergy => {
                        p.energy(dec)
                    },
            StandardCriteria::ECharge => {
                        p.e_charge(dec)
                    },
            StandardCriteria::BCharge => {
                        p.b_charge(dec)
                    },
            StandardCriteria::LCharge => {
                        p.l_charge(dec)
                    },
            StandardCriteria::FinCnt => {
                        if p.is_final(dec) {1.} else {0.}
                    },
            StandardCriteria::FinChargedCnt => {
                        if p.is_final(dec) && p.e_charge(dec) != 0. {1.} else {0.}
                    },
            StandardCriteria::PseudorapidityFilterCnt(mn, mx) => {
                        let pseudo = pseudorapidity(p.momentum(dec));
                        if p.e_charge(dec).abs() > 0.1 && (*mn <= pseudo && pseudo <= *mx) {
                            //println!(">>>{}", pseudo);
                            1.0
                        } else {
                            0.0
                        }
                    },
        }
    }

    fn name(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum VecCriteria {
    FinMomentum,
}

pub trait Particle {

    type Decoder;

    fn momentum_energy(&self, dec: &Self::Decoder) -> f64;

    fn momentum(&self, dec: &Self::Decoder) -> &(f64, f64, f64);

    fn mass_energy(&self, dec: &Self::Decoder) -> f64;

    fn energy(&self, dec: &Self::Decoder) -> f64 {
        ( self.mass_energy(dec).powi(2) + self.momentum_energy(dec).powi(2) ).sqrt()
    }

    /// Returns Electric charge
    fn e_charge(&self, dec: &Self::Decoder) -> f64;

    /// Returns Baryon charge
    fn b_charge(&self, dec: &Self::Decoder) -> f64;

    /// Returns Lepton charge
    fn l_charge(&self, dec: &Self::Decoder) -> f64;

    fn is_final(&self, dec: &Self::Decoder) -> bool;

    fn code(&self, dec: &Self::Decoder) -> i32;
}

pub trait HEPEvent {
    type P: Particle;
    fn particles(&self) -> impl Iterator<Item=&Self::P> + Clone;
}

pub struct HEPEventAnalyzer<'a, Event: HEPEvent> {
    events: &'a [Event],
}



#[derive(Debug, Clone, Default)]
pub struct ScalarAnalyzerResults(Vec<String>, Vec<Vec<f64>>);

impl ScalarAnalyzerResults {
    pub fn headers(&self) -> &Vec<String> {&self.0}
    pub fn values(&self) -> &Vec<Vec<f64>> {&self.1}
}

pub fn IS_FINAL_FILTER<'a, Event: HEPEvent>(x: &'a Event::P, dec: &<Event::P as Particle>::Decoder) -> bool { x.is_final(dec) }

impl<'a, Event: HEPEvent> HEPEventAnalyzer<'a, Event>
where &'a[Event]: rayon::iter::IntoParallelIterator<Item = &'a Event>
{
    pub fn new(events: &'a [Event]) -> Self { Self{events} }

    /// `criteria` - scalar criteria to calculate, calculated **after** filter
    pub fn calculate_criteria // <T: Sync + ScalarCriteria<'a, <Event::P as Particle>::Decoder, Event::P>>
    (   
            &self,
            filter: impl (Fn(&Event::P, &<Event::P as Particle>::Decoder) -> bool) + Sync,
            criteria: Vec<& (impl ScalarCriteria<'a, <Event::P as Particle>::Decoder, Event::P> +?Sized) >,//Vec<T>,
            dec: &<Event::P as Particle>::Decoder
    ) -> ScalarAnalyzerResults
    where
        <Event as HEPEvent>::P: 'static ,
        <Event::P as Particle>::Decoder: Sync
    {
        let headers = criteria.iter().map(|x| x.name() ).collect::<Vec<_>>();
        let criteria_vec = Arc::new(criteria);
        let dec = Arc::new(dec);
        let results = self.events.par_iter().map(
                |event| {
                    event.particles().filter(|x| {
                        filter(x, dec.as_ref())
                    }).fold(
                        criteria_vec.iter().map(|x| {(0., x)}).collect::<Vec<_>>(),
                        |mut crit, p| {
                            crit.iter_mut().for_each(
                                |x| {
                                    x.0 += x.1.get_criteria_value(&p, dec.as_ref())
                                }
                            );
                            crit
                        }
                    ).iter().map(|x| {x.0}).collect::<Vec<_>>()
                }
        ).collect::<Vec<_>>();
        ScalarAnalyzerResults(headers, results)   
    }

    pub fn calculate_distribution_criteria // <T: Sync + ScalarCriteria<'a, <Event::P as Particle>::Decoder, Event::P>>
    (   
            &self,
            filter: impl (Fn(&Event::P, &<Event::P as Particle>::Decoder) -> bool) + Sync,
            criteria: Vec<& (impl DistributionCritetia<'a, <Event::P as Particle>::Decoder, Event::P> +?Sized) >,//Vec<T>,
            dec: &<Event::P as Particle>::Decoder,
    ) -> Vec<(String, usize, Vec<(f64, f64)>, Vec<usize> )>
    where
        <Event as HEPEvent>::P: 'static ,
        <Event::P as Particle>::Decoder: Sync
    {
        let criteria_vec = Arc::new(criteria);
        let dec = Arc::new(dec);
        let results = self.events.par_iter()
        .fold(
            || criteria_vec.iter().map(
                |c|{
                    let n=c.get_bins().len();
                    (c, 0, vec![0usize; n])
                }
            ).collect::<Vec<(_, usize, Vec::<_>)>>(),
            |mut res, event| {
                let mut part = event.particles().filter(|x| filter(x, &dec));
                // criteria calculation for event
                res.iter_mut().for_each(
                    |(crit, n, bins)| {
                        // let (r_n, vs) = crit.get_criteria_values(
                        //     part.clone(), &dec
                        // );
                        let bs = crit.get_bins().len();
                        let mut r_n = 0;
                        part.clone().map(
                            |p| {
                                crit.get_criteria_bin_index(&p, &dec)
                            }
                        ).filter(
                            |&idx| {
                                idx >= 0 && (idx as usize) < bs
                            }
                        ).for_each(
                            |idx| {
                                r_n += 1;
                                bins[idx as usize] += 1;
                            }
                        );
                        *n += r_n;
                        // bins.iter_mut().zip(vs.iter()).for_each(
                        //     |(a, b)| {
                        //         *a += b;
                        //     }
                        // );
                    }
                );
                res
            }
        ).reduce(
            || criteria_vec.iter().map(
                    |c|{
                        let n=c.get_bins().len();
                        (c, 0, vec![0usize; n])
                    }
            ).collect::<Vec::<_>>(),
            |mut target, val| {
                target.iter_mut().zip(val.iter()).for_each(
                    |(a, b)| {
                        if a.0.name() != b.0.name() {
                            panic!("Wrong distribution ananlyze behaviour! Report this error");
                        }
                        a.1 += b.1;
                        a.2.iter_mut().zip(b.2.iter()).for_each(|(x, y)| *x += y );
                    }
                );
                target
            }
        ).iter().map(
            |(a, b, c)| {
                (a.name(), b.to_owned(), a.get_bins().to_vec(), c.to_owned())
            }
        ).collect::<Vec::<_>>();
        results
    }
}

/// Distribution criteria trait
/// use to calculate distribution of some event characteristics
/// for example, getting distribution of momentum of particles per each event
pub trait DistributionCritetia<'a, S, T>: Send + Sync//: PartialEq + Debug + Clone + Send
where T: Particle<Decoder = S> + 'static,
{
    /// get bins (HAVE TO BE SET BEFORE CALCULATING DISTRIBUTION)
    fn get_bins(&self) -> &[(f64, f64)];

    /// get for whole set of particles
    /// 
    /// returns (
    ///     .0: count of particles; 
    ///     .1: vec of len same as bins len, 
    ///         items are bin counters
    /// )
    // fn get_criteria_values(&self, ps: impl Iterator<Item=&'a T>, dec: &S) -> (usize, Vec<usize>);
    // where V: ;// Clone + Send + Sync + 'a; impl Iterator<Item=&Self::P> + Clone;
    
    /// get bin index for current particle
    /// return negative value if param of particle is less than minimum bin value
    fn get_criteria_bin_index(&self, p: &T, dec: &S) -> i64;// Clone + Send + Sync + 'a;

    fn name(&self) -> String;
}

///
pub struct ParticleCodeSelector {
    codes: HashSet<i64>
}


pub enum StandardDistributionCriteraDefiner<Event: HEPEvent> {

    /// distribution of particle momentum angle direction
    /// angle theta - between P and Z vectors
    /// bin : -pi, pi
    /// bin value: counter
    PdirTheta,
    /// pseudorapidity distribution
    PNu,
    PNu_selected(Vec<i32>),
    PTheta_selected(Vec<i32>),
    Custom(Box::<dyn (Fn(&Event::P, &<Event::P as Particle>::Decoder) -> f64) + Sync + Send>)
}

#[macro_export]
macro_rules! templated_std_crit_definer {
    /*(
        $g: ident::$i: ident,
        $($tmpl_in:tt)*
    ) => {
        $g::<$($tmpl_in)*>::$i 
    };*/

    (
        $g: ident::$i: ident $( ( $($arg:expr,)* ) )?,
        $($tmpl_in:tt)*
    ) => {
        $g::<$($tmpl_in)*>::$i $( ( $($arg,)* ) )?
    };
}

#[macro_export]
macro_rules! standard_criteria {
    /*(
        $Definer: ident::$DefinerVeriant: ident,
        $DataFile:path,
        $DEG_MIN:expr, $DEG_MAX:expr, $DEG_CNT:expr, $NAME:expr
    ) => {
        StandardDistributionCriteria::new(
            templated_std_crit_definer!(
                $Definer::$DefinerVeriant,
                <$DataFile as crate::fmt::generic::GenericDataContainer>::Block
            ),
            $DEG_MIN, $DEG_MAX, $DEG_CNT, $NAME
        )
    };*/

    (
        $Definer: ident::$DefinerVeriant: ident,
        $DataFile:path,
        $DEG_MIN:expr, $DEG_MAX:expr, $DEG_CNT:expr, $NAME:expr $(, arg=$($ARG:expr,)*)?
    ) => {
        StandardDistributionCriteria::new(
            templated_std_crit_definer!(
                $Definer::$DefinerVeriant $( ( $($ARG, )* ))?,
                <$DataFile as crate::fmt::generic::GenericDataContainer>::Block
            ),
            $DEG_MIN, $DEG_MAX, $DEG_CNT, $NAME
        )
    };
}

#[test]
fn test_dcrit_macro_creation() {
    /*let criteria = standard_criteria!(
        StandardDistributionCriteraDefiner::PdirTheta,
        PHQMDDataFile<'_>,
        0., 0., 0usize, "test".to_string()
    );*/
    /*let x = templated!(
        StandardDistributionCriteraDefiner::PdirTheta,
        <OSCEposDataFile<'_> as crate::fmt::generic::GenericDataContainer>::Block
    );*/
}


pub struct StandardDistributionCriteria<Event: HEPEvent> {
    definer: StandardDistributionCriteraDefiner<Event>,
    min: f64,
    max: f64,
    dx: f64,
    bins: Vec<(f64, f64)>,
    name: String,
}

impl<Event: HEPEvent> StandardDistributionCriteria<Event> {
    pub fn new(definer: StandardDistributionCriteraDefiner<Event>,
               min: f64, max: f64, bin_cnt: usize, name: String
    ) -> Self {
        let dx: f64 = (max - min) / (bin_cnt as f64);
        Self {
            definer,
            min,
            max,
            dx,
            bins: {
                (0..bin_cnt).map(
                    |i| {
                        let i = i as f64;
                        (min+dx*i, min+dx*(i+1.))
                    }
                ).collect()
            },
            name
        }
    }
}

#[test]
pub fn cuttest() {
    let s = StandardDistributionCriteria::new(StandardDistributionCriteraDefiner::PNu::<OSCEposBlock>, 0., PI+0.1, 2, "123".to_string());
    println!("{:?}", s.get_bins());
}

impl<'a, S, Event: HEPEvent> DistributionCritetia<'a, S, Event::P> for StandardDistributionCriteria<Event> 
    where Event::P: Particle<Decoder = S> + 'static,
{
    fn get_bins(&self) -> &[(f64, f64)] {
        &self.bins
    }

    /*fn get_criteria_values(&self, ps: impl Iterator<Item=&'a Event::P>, dec: &S) -> (usize, Vec<usize>)
    {
        let mut n = 0;
        let mut vals = vec![0; self.bins.len()];
        ps.into_iter().map(
            |p| {
                self.get_criteria_bin_index(&p, dec)
            }
        ).filter(
            |&idx| {
                idx >= 0 && (idx as usize) < self.bins.len()
            }
        ).for_each(
            |idx| {
                n += 1;
                vals[idx as usize] += 1;
            }
        );
        (n, vals)
    } */
   
    fn get_criteria_bin_index(&self, p: &Event::P, dec: &S) -> i64 {
        //let bin_cnt = self.bins.len();
        // let min = self.bins[0].0;
        // let max = self.bins[bin_cnt-1usize].1;
        // let dx: f64 = (min - max) / (bin_cnt as f64);
        let value = match &self.definer {
            StandardDistributionCriteraDefiner::PdirTheta => {
                let (x, y, z) = p.momentum(dec);
                ( (z) / ((x*x+y*y+z*z).sqrt()) ).acos()
            },
            StandardDistributionCriteraDefiner::PNu => {
                pseudorapidity(p.momentum(dec))
            },
            StandardDistributionCriteraDefiner::PNu_selected(v) => {
                if v.contains(&p.code(dec)) {
                    pseudorapidity(p.momentum(dec))
                } else {
                    self.max + self.dx + self.dx
                }
                
            }
            StandardDistributionCriteraDefiner::Custom(cst) => {
                cst.as_ref()(p, dec)
            },
            StandardDistributionCriteraDefiner::PTheta_selected(v) => {
                if v.contains(&p.code(dec)) {
                    let (x, y, z) = p.momentum(dec);
                    ( (z) / ((x*x+y*y+z*z).sqrt()) ).acos()
                } else {
                    // println!("[WARNING] {:?} out of context", p);
                    self.max + self.dx + self.dx
                }
                
            }
        };
        // println!(">>>{} :: {}", value, ((value-self.min) / self.dx).ceil() as i64);
        ((value-self.min) / self.dx).ceil() as i64
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}