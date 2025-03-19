use rayon::prelude::*;
use std::{borrow::Borrow, collections::HashSet, fmt::Debug, sync::Arc};

pub trait ScalarCriteria<'a, S, T>: Sized + PartialEq + Debug + Clone + Send
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
    PseudorapidityFilterCnt(f64, f64),
}


/*

    
def pseudorapidity(v1, v2):
    theta = np.arccos(v1.scal(v2) / (v1.len() * v2.len()))
    return -np.log(np.tan(theta/2))

def zpseudorapidity(v1):
    # return abs(np.arctanh(v1.z / v1.len()))
    return pseudorapidity(v1, Vec(0, 0, 1))

*/

pub fn pseudorapidity((x, y, z): &(f64, f64, f64)) -> f64 {
    let theta = ( (z) / ((x*x+y*y+z*z).sqrt()) ).acos();
    - (theta / 2.0).tan().ln()
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
        (self.mass_energy(dec).powi(2) + self.momentum_energy(dec).powi(2)).sqrt()
    }

    /// Returns Electric charge
    fn e_charge(&self, dec: &Self::Decoder) -> f64;

    /// Returns Baryon charge
    fn b_charge(&self, dec: &Self::Decoder) -> f64;

    /// Returns Lepton charge
    fn l_charge(&self, dec: &Self::Decoder) -> f64;

    fn is_final(&self, dec: &Self::Decoder) -> bool;
}

pub trait HEPEvent {
    type P: Particle;
    fn particles(&self) -> impl Iterator<Item=&Self::P> + Clone;
}

pub struct HEPEventAnalyzer<'a, Event: HEPEvent> {
    events: &'a [Event],
}



#[derive(Debug, Clone)]
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

    ///
    /// `criteria` - scalar criteria to calculate, calculated **after** filter
    pub fn calculate_criteria<T: Sync + ScalarCriteria<'a, <Event::P as Particle>::Decoder, Event::P>>
    (   
            &self,
            filter: impl (Fn(&Event::P, &<Event::P as Particle>::Decoder) -> bool) + Sync,
            criteria: Vec<T>, dec: &<Event::P as Particle>::Decoder
    ) -> ScalarAnalyzerResults
    where
        <Event as HEPEvent>::P: 'static ,
        <Event::P as Particle>::Decoder: Sync
    {
        // if !vec_criteria.is_empty() {
        //     panic!("NOT IMPLEMENTED YET")
        // }
        //let criteria_vec = criteria.iter().map(|x| {(x.get_calculer(), 0f64)}).collect::<Vec<_>>();
        //let criteria_vec = criteria.iter().map(|x| {(x.get_calculer(), 0f64)}).collect::<Vec<_>>();
        let headers = criteria.iter().map(|x| x.name() ).collect::<Vec<_>>();

        let criteria_vec = Arc::new(criteria);
        //let filter = Arc::new(filter);
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
}