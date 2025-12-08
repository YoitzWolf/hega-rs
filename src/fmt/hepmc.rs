use std::error::Error;
use std::io::prelude::*;
use with_position::{WithPosition, Position};

use super::decoder::EposDict;
/// OSCAR1999 format reader and interpreter

use super::generic::*;

#[derive(Debug)]
pub struct HepMCBlock {
    /// Block Header
    pub header: HepMCBlockHeader,
    /// Particles
    pub event: Vec<HepMCParticle>
}

#[derive(Debug)]
pub struct HepMCBlockHeader {
    pub event_id: usize,
    /// Count of particles in OSCAR event description
    pub nout: usize,

    /// weight
    pub weight: f64,
    
}

#[derive(Debug)]
pub struct HepMCHeader {
    // Collision Energy
    //pub snn: f64,
    // OSCAR event signature. 
    // (1, 1) + (1, 1) or Au(..) + Au(..)
    // pub event_signature: String,
}


/*
    Based on HepMC \\
     // NOTE: Keep in mind the data alignment \\
    `struct GenParticleData {\\
        int        pid;               ///< PDG ID\\
        int        status;            ///< Status\\
        bool       is_mass_set;       ///< Check if generated mass is set\\
        double     mass;              ///< Generated mass (if set)\\
        FourVector momentum;          ///< Momentum\\
    };`
*/
#[derive(Debug)]
pub struct HepMCParticle {
    pub code: i32,
    pub status: i32,
    pub mass: f64,
    pub energy: f64,
    pub p: (f64, f64, f64)
}

impl<'a> TryFrom<(HepMCBlockHeader, &'a Vec<String>)> for HepMCBlock {
    type Error = Box<dyn Error>;
    fn try_from(value: (HepMCBlockHeader, &'a Vec<String>)) -> Result<Self, Self::Error> {
        let particles = value.1.iter().try_fold(
            vec![],
            |mut v, line| -> Result<Vec<_>, Box<dyn Error>> {
                let mut iter = line.split_ascii_whitespace();
                let mut args = [""; 10];
                [0usize, 1, 2, 3, 4, 5 ,6 ,7 ,8 ,9].iter()
                    .try_fold(&mut args, |a, x| {
                        let v = iter.next();
                        a[*x] = v?;
                        Some(a)
                });
                v.push(
                    HepMCParticle::try_from( args )?
                );
                Ok(v)
            }
        )?;
        Ok(
            Self {
                header: value.0,
                event: particles,
            }
        )
    }
}

impl<'a> TryFrom<[&'a str; 10]> for HepMCParticle {
    type Error = Box<dyn Error>;

    fn try_from(value: [&'a str; 10]) -> Result<Self, Self::Error> {
        // value[0] is P literal
        let s = Self {
            code: value[3].parse().unwrap(),
            p: (value[4].parse().unwrap(), value[5].parse().unwrap(), value[6].parse().unwrap()),
            energy: value[7].parse().unwrap(),
            mass: value[8].parse().unwrap(),
            status: value[9].parse().unwrap(),
        };
        Ok(s)
    }
}

impl<'a> DataBlock<'a, HepMCBlockHeader> for HepMCBlock {
    fn get_header(&self) -> &HepMCBlockHeader {
        &self.header
    }
}


#[derive(Debug)]
pub struct HepMCDataFile<'a> {
    header: HepMCHeader,
    events: Vec<HepMCBlock>,
    pub decoder: &'a EposDict
}


impl<'a, 'b> GenericDataContainer<'a, 'b> for HepMCDataFile<'b> {
    type Header = HepMCHeader;

    type BlockHeader = HepMCBlockHeader;

    type Block = HepMCBlock;

    type Decoder = EposDict;

    fn get_header(&self) -> &Self::Header {
        &self.header
    }

    fn get_blocks(&self) -> &Vec<Self::Block> {
        &self.events
    }

    fn borrow_blocks(self)  -> Vec<Self::Block> {
        self.events
    }

    fn upload<T: Sized + std::io::Read>(data: std::io::BufReader<T>, decoder: &'b Self::Decoder) -> Result<Self, std::io::Error> {
        match data.lines().with_position().enumerate().try_fold(
            (
                None,
                None,
                Vec::<String>::new(),
                vec![]
            ),
            |
                (mut header, mut bufheader, mut buf, mut events)
                , (idx, (position, _line))
            | -> Result<_, Box<dyn Error> > {
                match _line {
                    Ok(line) => {
                        let line = line.trim().to_string();
                        if line.starts_with("HepMC") {
                            if let Position::Last = position {
                                if let Some(hd) = bufheader {
                                    let block = Self::Block::try_from(
                                        (hd, &buf)
                                    )?;
                                    events.push(block);
                                    buf.clear();
                                }
                                bufheader = None;
                            }
                            Ok((Some(HepMCHeader {}), bufheader, buf, events))
                        } else if line.starts_with("E") {
                            // new event
                            if let Some(hd) = bufheader {
                                let block = Self::Block::try_from(
                                    (hd, &buf)
                                )?;
                                events.push(block);
                                buf.clear();
                            }
                            let toks: Vec<_> = line.split_ascii_whitespace().filter(|x| x.len() > 0).collect();
                            bufheader = Some(Self::BlockHeader {                                        
                                weight: 0.0,
                                event_id: toks[1].parse()?,
                                nout: toks[3].parse()?,
                            });
                            Ok((header, bufheader, buf, events))
                        } else if line.starts_with("W") {
                            if let Some(hd) = &mut bufheader {
                                let tr = line.split_ascii_whitespace().filter(|x| x.len() > 0).collect::<Vec<&str>>();
                                hd.weight = tr[1].parse()?;
                            }
                            Ok((header, bufheader, buf, events))
                        } else if line.starts_with("P") {
                            // particle!
                            buf.push(line);
                            if let Position::Last = position {
                                if let Some(hd) = bufheader {
                                    let block = Self::Block::try_from(
                                        (hd, &buf)
                                    )?;
                                    events.push(block);
                                    buf.clear();
                                }
                                bufheader = None;
                            }
                            Ok((header, bufheader, buf, events))
                        } else {
                            if let Position::Last = position {
                                if let Some(hd) = bufheader {
                                    let block = Self::Block::try_from(
                                        (hd, &buf)
                                    )?;
                                    events.push(block);
                                    buf.clear();
                                }
                                bufheader = None;
                            }
                            Ok((header, bufheader, buf, events))
                        }
                    },
                    Err(e) => {
                        Err(Box::new(e))
                    },
                }
            }
        ) {
            Ok((a, b ,c, events)) => {
                Ok(
                    Self {
                        header: a.unwrap(),
                        events: events,
                        decoder: decoder
                    }
                )
            },
            Err(e) => {
                Err(std::io::Error::new::<String>(std::io::ErrorKind::InvalidData, e.as_ref().to_string().into()))
            },
        }
    }

    fn push_back(&mut self, mut t: Self) {
        self.events.append(&mut t.events);
    }
}
