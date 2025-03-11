use rayon::prelude::*;
use serde::Serialize;
use std::{borrow::Borrow, collections::HashSet, fmt::Debug, sync::Arc};

pub trait ScalarCriteria<T: Particle>: Sized + PartialEq + Debug + Clone + Send {
    fn get_calculer(&self) -> impl (Fn(&T) -> f64) + Clone + Send + Sync + 'static;

    fn name(&self) -> String;
}

#[derive(Debug, PartialEq, Clone)]
pub enum StandardCriteria {
    FinEnergy,
    ECharge,
    BCharge,
    LCharge,
    FinCnt,
    PseudorapidityFilterCnt(f64, f64),
}

impl<T: Particle + 'static> ScalarCriteria<T> for StandardCriteria {
    fn get_calculer(&self) -> impl (Fn(&T) -> f64) + Clone + Send + 'static{
        match self {
            StandardCriteria::FinEnergy => {
                |p: &T| -> f64 {p.energy()}
            },
            StandardCriteria::ECharge => {
                |p: &T| -> f64 {p.e_charge()}
            },
            StandardCriteria::BCharge => {
                |p: &T| -> f64 {p.b_charge()}
            },
            StandardCriteria::LCharge => {
                |p: &T| -> f64 {p.l_charge()}
            },
            StandardCriteria::FinCnt => {
                |p: &T| -> f64 { if p.is_final() {1.} else {0.} }
            },
            StandardCriteria::PseudorapidityFilterCnt(mn, mx) => {
                |p: &T| -> f64 {
                    if p.is_final() {1.} else {0.}
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
    fn momentum_energy(&self) -> f64;
    fn mass_energy(&self) -> f64;

    fn energy(&self) -> f64 {
        self.mass_energy() + self.momentum_energy()
    }

    /// Returns Electric charge
    fn e_charge(&self) -> f64;

    /// Returns Baryon charge
    fn b_charge(&self) -> f64;

    /// Returns Lepton charge
    fn l_charge(&self) -> f64;

    fn is_final(&self) -> bool;
}

pub trait HEPEvent {
    type P: Particle;
    fn particles(&self) -> impl Iterator<Item=Self::P> + Clone;
}

pub struct HEPEventAnalyzer<'a, Event: HEPEvent> {
    events: &'a [Event],
}

#[derive(Debug, Clone, Serialize)]
pub struct ScalarAnalyzerResults(Vec<String>, Vec<Vec<f64>>);

impl ScalarAnalyzerResults {
    pub fn headers(&self) -> &Vec<String> {&self.0}
    pub fn values(&self) -> &Vec<Vec<f64>> {&self.1}
}


impl<'a, Event: HEPEvent> HEPEventAnalyzer<'a, Event>
where &'a[Event]: rayon::iter::IntoParallelIterator<Item = &'a Event>
{

    pub fn new(events: &'a [Event]) -> Self { Self{events} }

    ///
    /// `criteria` - scalar criteria to calculate, calculated **after** filter
    pub fn calculate_criteria<T: ScalarCriteria<Event::P>>(&self, filter: impl (Fn(&Event::P) -> bool), criteria: Vec<T>) -> ScalarAnalyzerResults {
        // if !vec_criteria.is_empty() {
        //     panic!("NOT IMPLEMENTED YET")
        // }
        //let criteria_vec = criteria.iter().map(|x| {(x.get_calculer(), 0f64)}).collect::<Vec<_>>();
        let criteria_vec = criteria.iter().map(|x| {(x.get_calculer(), 0f64)}).collect::<Vec<_>>();
        let headers = criteria.iter().map(|x| x.name() ).collect::<Vec<_>>();

        let criteria_vec = Arc::new(criteria_vec);

        let results = self.events.par_iter().map(
                |event| {
                    event.particles().fold(
                        criteria_vec.as_ref().clone(), |mut crit, ev| {
                            crit.iter_mut().for_each(|x| {x.1 += x.0(&ev)} );
                            crit
                        }
                    ).iter().map(|x| {x.1}).collect::<Vec<_>>()
                }
        ).collect::<Vec<_>>();
        ScalarAnalyzerResults(headers, results)   
    }
}