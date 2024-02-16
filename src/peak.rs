//! A peak is the most atomic unit of a (processed) mass spectrum. It represents
//! a location in one (or more) coordinate spaces with an measured intensity value
//!
//! All peak-like types implement [`PartialEq`], [`PartialOrd`], and [`CoordinateLike`]
//!

use std::cmp;
use std::fmt;

use crate::coordinate::{CoordinateLike, IndexType, IndexedCoordinate, Mass, MZ};
use crate::{implement_centroidlike_inner, implement_deconvoluted_centroidlike_inner};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An intensity measurement is an entity that has a measured intensity
/// of whateger it is.
pub trait IntensityMeasurement {
    fn intensity(&self) -> f32;
}

pub trait IntensityMeasurementMut: IntensityMeasurement {
    fn intensity_mut(&mut self) -> &mut f32;
}

impl<T: IntensityMeasurement> IntensityMeasurement for &T {
    fn intensity(&self) -> f32 {
        (*self).intensity()
    }
}

impl<T: IntensityMeasurement> IntensityMeasurement for &mut T {
    fn intensity(&self) -> f32 {
        IntensityMeasurement::intensity(*self)
    }
}

impl<T: IntensityMeasurementMut> IntensityMeasurementMut for &mut T {
    fn intensity_mut(&mut self) -> &mut f32 {
        IntensityMeasurementMut::intensity_mut(*self)
    }
}

/// A [`CentroidLike`] entity is indexed in m/z coordinate space and
/// is an [`IntensityMeasurement`]
pub trait CentroidLike: IndexedCoordinate<MZ> + IntensityMeasurement {
    #[inline]
    fn as_centroid(&self) -> CentroidPeak {
        CentroidPeak {
            mz: self.coordinate(),
            intensity: self.intensity(),
            index: self.get_index(),
        }
    }
}

/// A known charge has a determined charge state value
pub trait KnownCharge {
    fn charge(&self) -> i32;
}

impl<T: KnownCharge> KnownCharge for &T {
    fn charge(&self) -> i32 {
        (*self).charge()
    }
}

pub trait KnownChargeMut: KnownCharge {
    fn charge_mut(&mut self) -> &mut i32;
}

impl<T: KnownCharge> KnownCharge for &mut T {
    fn charge(&self) -> i32 {
        KnownCharge::charge(*self)
    }
}

impl<T: KnownChargeMut> KnownChargeMut for &mut T {
    fn charge_mut(&mut self) -> &mut i32 {
        KnownChargeMut::charge_mut(*self)
    }
}

/// A [`DeconvolutedCentroidLike`] entity is indexed in the neutral mass
/// coordinate space, has known charge state and an aggregated intensity
/// measurement. Any [`DeconvolutedCentroidLike`] can be converted into
/// a [`DeconvolutedPeak`]
pub trait DeconvolutedCentroidLike:
    IndexedCoordinate<Mass> + IntensityMeasurement + KnownCharge
{
    fn as_centroid(&self) -> DeconvolutedPeak {
        DeconvolutedPeak {
            neutral_mass: self.coordinate(),
            intensity: self.intensity(),
            charge: self.charge(),
            index: self.get_index(),
        }
    }
}

/// Represent a single m/z coordinate with an
/// intensity and an index. Nearly the most basic
/// peak representation for peak-picked data.
#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CentroidPeak {
    pub mz: f64,
    pub intensity: f32,
    pub index: IndexType,
}

impl CentroidPeak {
    #[inline]
    pub fn new(mz: f64, intensity: f32, index: IndexType) -> CentroidPeak {
        CentroidPeak {
            mz,
            intensity,
            index,
        }
    }
}

impl fmt::Display for CentroidPeak {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CentroidPeak({}, {}, {})",
            self.mz, self.intensity, self.index
        )
    }
}

implement_centroidlike_inner!(CentroidPeak, true, false);

impl<T: IndexedCoordinate<MZ> + IntensityMeasurement> CentroidLike for T {}

impl<T: IndexedCoordinate<Mass> + IntensityMeasurement + KnownCharge> DeconvolutedCentroidLike
    for T
{
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MZPoint {
    pub mz: f64,
    pub intensity: f32,
}

impl MZPoint {
    #[inline]
    pub fn new(mz: f64, intensity: f32) -> MZPoint {
        MZPoint { mz, intensity }
    }
}

impl CoordinateLike<MZ> for MZPoint {
    fn coordinate(&self) -> f64 {
        self.mz
    }
}

impl IntensityMeasurement for MZPoint {
    #[inline]
    fn intensity(&self) -> f32 {
        self.intensity
    }
}

impl IntensityMeasurementMut for MZPoint {
    #[inline]
    fn intensity_mut(&mut self) -> &mut f32 {
        &mut self.intensity
    }
}

impl IndexedCoordinate<MZ> for MZPoint {
    #[inline]
    fn get_index(&self) -> IndexType {
        0
    }
    fn set_index(&mut self, _index: IndexType) {}
}

impl From<MZPoint> for CentroidPeak {
    fn from(peak: MZPoint) -> Self {
        CentroidPeak {
            mz: peak.mz,
            intensity: peak.intensity,
            index: 0,
        }
    }
}

impl From<CentroidPeak> for MZPoint {
    fn from(peak: CentroidPeak) -> Self {
        let mut inst = Self {
            mz: peak.coordinate(),
            intensity: peak.intensity(),
        };
        inst.set_index(peak.index);
        inst
    }
}

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Represent a single neutral mass coordinate with an
/// intensity, a known charge and an index.
pub struct DeconvolutedPeak {
    pub neutral_mass: f64,
    pub intensity: f32,
    pub charge: i32,
    pub index: IndexType,
}

impl DeconvolutedPeak {
    pub fn new(neutral_mass: f64, intensity: f32, charge: i32, index: IndexType) -> Self {
        Self {
            neutral_mass,
            intensity,
            charge,
            index,
        }
    }

    pub fn mz(&self) -> f64 {
        let charge_carrier: f64 = 1.007276;
        let charge = self.charge as f64;
        (self.neutral_mass + charge_carrier * charge) / charge
    }
}

implement_deconvoluted_centroidlike_inner!(DeconvolutedPeak, true, false);

impl fmt::Display for DeconvolutedPeak {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DeconvolutedPeak({}, {}, {}, {})",
            self.neutral_mass, self.intensity, self.charge, self.index
        )
    }
}

impl CoordinateLike<MZ> for DeconvolutedPeak {
    fn coordinate(&self) -> f64 {
        self.mz()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::coordinate::*;

    #[test]
    fn test_conversion() {
        let x = CentroidPeak::new(204.07, 5000f32, 19);
        let y: MZPoint = x.clone().into();
        assert_eq!(y.mz, x.mz);
        assert_eq!(y.coordinate(), x.coordinate());
        assert_eq!(y.intensity(), x.intensity());
        // MZPoint doesn't use index
        let z: CentroidPeak = y.clone().into();
        assert_eq!(z, y);
    }

    #[test]
    fn test_coordinate_context() {
        let x = DeconvolutedPeak {
            neutral_mass: 799.359964027,
            charge: 2,
            intensity: 5000f32,
            index: 1,
        };
        assert_eq!(x.neutral_mass, 799.359964027);
        assert_eq!(CoordinateLike::<Mass>::coordinate(&x), 799.359964027);
        assert_eq!(Mass::coordinate(&x), 799.359964027);
        assert!((x.mz() - 400.68725848027003).abs() < 1e-6);
        assert!((MZ::coordinate(&x) - 400.68725848027003).abs() < 1e-6);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize() -> std::io::Result<()> {
        use serde_json;
        use std::io;
        use std::io::prelude::*;

        let mut buff = Vec::new();
        let buffer_writer = io::Cursor::new(&mut buff);
        let mut writer = io::BufWriter::new(buffer_writer);

        let x = CentroidPeak::new(204.07, 5000f32, 19);
        // let y: MZPoint = x.clone().into();
        serde_json::to_writer_pretty(&mut writer, &x)?;
        writer.flush()?;
        let view = String::from_utf8_lossy(writer.get_ref().get_ref());
        let peak: CentroidPeak = serde_json::from_str(&view)?;
        assert!((peak.mz() - x.mz()).abs() < 1e-6);
        Ok(())
    }
}
