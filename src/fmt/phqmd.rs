use std::error::Error;
use std::io::prelude::*;

use super::decoder::EposDict;
/// PHQMD format reader and interpreter

use super::generic::*;

#[derive(Debug)]
pub struct PHQMDBlock {
    /// Block Header
    pub header: PHQMDBlockHeader,
    /// Particles
    pub event: Vec<PHQMDParticle>
}

#[derive(Debug)]
pub struct PHQMDBlockHeader {
    /// Count of particles in OSCAR event description
    pub nout: usize
}

#[derive(Debug)]
pub struct PHQMDHeader {
}

#[derive(Debug)]
pub struct PHQMDDataFile<'a> {
    header: PHQMDHeader,
    events: Vec<PHQMDBlock>,
    pub decoder: &'a EposDict
}

#[derive(Debug)]
pub struct PHQMDParticle {
    pub code: i32,
    pub charge: i32,
    pub p: (f64, f64, f64),
    pub E: f64,
    pub id: usize
}


impl<'a> TryFrom<[&'a str; 9]> for PHQMDParticle {
    type Error = Box<dyn Error>;

    fn try_from(value: [&'a str; 9]) -> Result<Self, Self::Error> {
        let s = Self {
            
            code:   value[0].parse()?,
            charge: value[1].parse()?,
            p:      (value[2].parse()?, value[3].parse()?, value[4].parse()?),
            E:      value[5].parse()?,
            id:     value[8].parse()?,
        };
        Ok(s)
    }
}

impl<'a> TryFrom<(PHQMDBlockHeader, &'a Vec<String>)> for PHQMDBlock {
    type Error = Box<dyn Error>;
    fn try_from(value: (PHQMDBlockHeader, &'a Vec<String>)) -> Result<Self, Self::Error> {
        let particles = value.1.iter().try_fold(
            vec![],
            |mut v, line| -> Result<Vec<_>, Box<dyn Error>> {
                let mut iter = line.split_ascii_whitespace();
                let mut args = [""; 9];
                [0usize, 1, 2, 3, 4, 5 ,6 ,7 ,8].iter()
                    .try_fold(&mut args, |a, x| {
                        let v = iter.next();
                        a[*x] = v?;
                        Some(a)
                });
                v.push(
                    PHQMDParticle::try_from( args )?
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

impl<'a> DataBlock<'a, PHQMDBlockHeader> for PHQMDBlock {
    fn get_header(&self) -> &PHQMDBlockHeader {
        &self.header
    }
}

impl<'a, 'b> GenericDataContainer<'a, 'b> for PHQMDDataFile<'b> {
    type Header = PHQMDHeader;

    type BlockHeader = PHQMDBlockHeader;

    type Block = PHQMDBlock;

    type Decoder = EposDict;

    fn get_header(&self) -> &Self::Header {
        &self.header
    }

    fn get_blocks(&self) -> &Vec<Self::Block> {
        &self.events
    }

    fn upload<T: Sized + std::io::Read>(data: std::io::BufReader<T>, decoder: &'b Self::Decoder) -> Result<Self, std::io::Error> {

        let mut dit = data.lines().enumerate();

        let header = Some (
            PHQMDHeader {}
        );
        match dit.try_fold(
            (
                false,
                None,
                Vec::<String>::new(),
                vec![]
            ),
            |
                ( mut skip, mut bufheader, mut buf, mut events)
                , (i, _line)| -> Result<_, Box<dyn Error> > {
                match _line {
                    Ok(line) => {
                        if (skip) { Ok((false, bufheader, buf, events)) } else
                        {
                            // DATA OR EMPTY
                            let tr = line.trim();
                            let tokens: Vec<_> = tr.split_ascii_whitespace().collect();
                            match tokens.len() {
                                5 => {
                                    // new event
                                    if let Some(hd) = bufheader {
                                        let block = PHQMDBlock::try_from(
                                            (hd, &buf)
                                        )?;
                                        events.push(block);
                                        buf.clear();
                                    }
                                    bufheader = Some(PHQMDBlockHeader {
                                        nout: tokens.first().unwrap().parse().unwrap(),
                                    });
                                    skip = true;
                                },
                                (6..) => {
                                    // line event
                                    buf.push(line);
                                },
                                _ => {
                                    // warn!("Bad line!");

                                }
                            }
                            Ok((skip, bufheader, buf, events))
                        }
                    },
                    Err(e) => {
                        Err(Box::new(e))
                    },
                }
            }
        ) {
            Ok((_skip, _a, _b , events)) => {
                Ok(
                    Self {
                        header: header.unwrap(),
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
}