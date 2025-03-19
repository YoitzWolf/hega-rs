use std::error::Error;
use std::io::prelude::*;

use super::decoder::EposDict;
/// OSCAR1999 format reader and interpreter

use super::generic::*;

#[derive(Debug)]
pub struct OSCEposBlock {
    /// Block Header
    pub header: OSCEposBlockHeader,
    /// Particles
    pub event: Vec<OscarParticle>
}

#[derive(Debug)]
pub struct OSCEposBlockHeader {
    /// Count of particles in OSCAR event description
    pub nout: usize
}

#[derive(Debug)]
pub struct OSCEposHeader {
    /// Collision Energy
    pub snn: f64,
    /// OSCAR event signature. 
    /// (1, 1) + (1, 1) or Au(..) + Au(..)
    pub event_signature: String,
}

#[derive(Debug)]
pub struct OSCEposDataFile<'a> {
    header: OSCEposHeader,
    events: Vec<OSCEposBlock>,
    pub decoder: &'a EposDict
}

#[derive(Debug)]
pub struct OscarParticle {
    pub id: usize,
    pub code: i32,
    pub state: i32,
    pub p: (f64, f64, f64),
    pub p0: f64,
    pub mass: f64,
    pub coords: (f64, f64, f64),
    pub time: f64,

    //pub echarge: f64,
    //pub contains: Option<Vec<OscarParticle>>

}


impl<'a> TryFrom<[&'a str; 12]> for OscarParticle {
    type Error = Box<dyn Error>;

    fn try_from(value: [&'a str; 12]) -> Result<Self, Self::Error> {
        let s = Self {
            id:     value[0].parse()?,
            code:   value[1].parse()?,
            state:  value[2].parse()?,
            p:      (value[3].parse()?, value[4].parse()?, value[5].parse()?),
            p0:     value[6].parse()?,
            mass:   value[7].parse()?,
            coords: (value[8].parse()?, value[9].parse()?, value[10].parse()?),
            time:   value[11].parse()?,
            
            //echarge: 0.,
            //contains: None,
        };
        Ok(s)
    }
}

impl<'a> TryFrom<(OSCEposBlockHeader, &'a Vec<String>)> for OSCEposBlock {
    type Error = Box<dyn Error>;
    fn try_from(value: (OSCEposBlockHeader, &'a Vec<String>)) -> Result<Self, Self::Error> {
        let particles = value.1.iter().try_fold(
            vec![],
            |mut v, line| -> Result<Vec<_>, Box<dyn Error>> {
                let mut iter = line.split_ascii_whitespace();
                let mut args = [""; 12];
                [0usize, 1, 2, 3, 4, 5 ,6 ,7 ,8 ,9 ,10, 11].iter()
                    .try_fold(&mut args, |a, x| {
                        let v = iter.next();
                        a[*x] = v?;
                        Some(a)
                });
                v.push(
                    OscarParticle::try_from( args )?
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

impl<'a> DataBlock<'a, OSCEposBlockHeader> for OSCEposBlock {
    fn get_header(&self) -> &OSCEposBlockHeader {
        &self.header
    }
}

impl<'a, 'b> GenericDataContainer<'a, 'b> for OSCEposDataFile<'b> {
    type Header = OSCEposHeader;

    type BlockHeader = OSCEposBlockHeader;

    type Block = OSCEposBlock;

    type Decoder = EposDict;

    fn get_header(&self) -> &Self::Header {
        &self.header
    }

    fn get_blocks(&self) -> &Vec<Self::Block> {
        &self.events
    }

    fn upload<T: Sized + std::io::Read>(data: std::io::BufReader<T>, decoder: &'b Self::Decoder) -> Result<Self, std::io::Error> {
        match data.lines().try_fold(
            (
                None,
                None,
                Vec::<String>::new(),
                vec![]
            ),
            |
                (mut header, mut bufheader, mut buf, mut events)
                , _line| -> Result<_, Box<dyn Error> > {
                match _line {
                    Ok(line) => {
                        if line.starts_with("#") {
                            // COMMENT
                            let cmt = line.strip_prefix("#").unwrap();
                            let s: Vec<_> = cmt.split("nncm").collect();
                            if s.len() < 2 {
                                // SKIP
                                Ok((header, bufheader, buf, events))
                            } else {
                                header = Some (
                                    OSCEposHeader {
                                        snn: s[1].trim().split_ascii_whitespace().collect::<Vec<_>>().first().unwrap().parse()?,
                                        event_signature: s[0].trim().to_owned(),
                                    }
                                );
                                Ok((header, bufheader, buf, events))
                            }
                            // END COMMENT
                        } else {
                            // DATA OR EMPTY
                            let tr = line.trim();
                            let tokens: Vec<_> = tr.split_ascii_whitespace().collect();
                            match tokens.len() {
                                5 => {
                                    // new event
                                    if let Some(hd) = bufheader {
                                        let block = OSCEposBlock::try_from(
                                            (hd, &buf)
                                        )?;
                                        events.push(block);
                                        buf.clear();
                                    }
                                    bufheader = Some(OSCEposBlockHeader {
                                        nout: tokens.first().unwrap().parse().unwrap(),
                                    });
                                },
                                (6..) => {
                                    // line event
                                    buf.push(line);
                                },
                                _ => {
                                    // warn!("Bad line!");

                                }
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



/* --------------------------------- URQMD --------------------------------------- */



/// OSC1997A UrQMD format reader (.f19 UrQMD output)
/// reuses OSCEposDataFile blocks and headers as much as possible
/// uses OscarParticle directly
/// sets STATUS code to =0 to all particles
/// 
#[derive(Debug)]
pub struct OSC97UrQMDDataFile<'a> {
    header: OSCEposHeader,
    events: Vec<OSCEposBlock>,
    pub decoder: &'a EposDict
}


impl<'a, 'b> GenericDataContainer<'a, 'b> for OSC97UrQMDDataFile<'b> {
    type Header = OSCEposHeader;

    type BlockHeader = OSCEposBlockHeader;

    type Block = OSCEposBlock;

    type Decoder = EposDict;

    fn get_header(&self) -> &Self::Header {
        &self.header
    }

    fn get_blocks(&self) -> &Vec<Self::Block> {
        &self.events
    }

    fn upload<T: Sized + std::io::Read>(data: std::io::BufReader<T>, decoder: &'b Self::Decoder) -> Result<Self, std::io::Error> {
        match data.lines().enumerate().try_fold(
            (
                None,
                None,
                Vec::<String>::new(),
                vec![]
            ),
            |
                (mut header, mut bufheader, mut buf, mut events)
                , (idx, _line)| -> Result<_, Box<dyn Error> > {
                match _line {
                    Ok(line) => {
                        if idx <= 2 && line.contains("UrQMD") {
                            // COMMENT
                            let cmt = line.trim();
                            let s: Vec<_> = line.split_ascii_whitespace().map(|x| x.trim()).filter(|x| x.len()>0).collect();
                            if s.len() < 2 {
                                // SKIP
                                Ok((header, bufheader, buf, events))
                            } else {
                                header = Some (
                                    Self::Header {
                                        snn: s.last().unwrap().parse()?,
                                        event_signature: cmt.to_owned(),
                                    }
                                );
                                Ok((header, bufheader, buf, events))
                            }
                            // END COMMENT
                        } else {
                            // DATA OR EMPTY
                            let tr = line.trim();
                            let mut tokens: Vec<_> = tr.split_ascii_whitespace().filter(|x| x.len() > 0).collect();
                            match tokens.len() {
                                4 => {
                                    // new event
                                    if let Some(hd) = bufheader {
                                        let block = Self::Block::try_from(
                                            (hd, &buf)
                                        )?;
                                        events.push(block);
                                        buf.clear();
                                    }
                                    bufheader = Some(Self::BlockHeader {
                                        nout: tokens[1].parse().unwrap(),
                                    });
                                },
                                (5..) => {
                                    // line event
                                    buf.push({
                                        tokens.insert(2, &"0"); // add STATUS code, bcs .f19 OSCAR1997A UrQMD output files dont do this
                                        tokens.join(" ")
                                    });
                                },
                                _ => {
                                    // warn!("Bad line!");
                                }
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
