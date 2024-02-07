// automatically generated by the FlatBuffers compiler, do not modify


// @generated

use core::mem;
use core::cmp::Ordering;

extern crate flatbuffers;
use self::flatbuffers::{EndianScalar, Follow};

pub enum SpectraDetectorMappingOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct SpectraDetectorMapping<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for SpectraDetectorMapping<'a> {
  type Inner = SpectraDetectorMapping<'a>;
  #[inline]
  unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    Self { _tab: flatbuffers::Table::new(buf, loc) }
  }
}

impl<'a> SpectraDetectorMapping<'a> {
  pub const VT_SPECTRUM: flatbuffers::VOffsetT = 4;
  pub const VT_DETECTOR_ID: flatbuffers::VOffsetT = 6;
  pub const VT_N_SPECTRA: flatbuffers::VOffsetT = 8;

  #[inline]
  pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
    SpectraDetectorMapping { _tab: table }
  }
  #[allow(unused_mut)]
  pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
    _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
    args: &'args SpectraDetectorMappingArgs<'args>
  ) -> flatbuffers::WIPOffset<SpectraDetectorMapping<'bldr>> {
    let mut builder = SpectraDetectorMappingBuilder::new(_fbb);
    builder.add_n_spectra(args.n_spectra);
    if let Some(x) = args.detector_id { builder.add_detector_id(x); }
    if let Some(x) = args.spectrum { builder.add_spectrum(x); }
    builder.finish()
  }


  #[inline]
  pub fn spectrum(&self) -> Option<flatbuffers::Vector<'a, i32>> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, i32>>>(SpectraDetectorMapping::VT_SPECTRUM, None)}
  }
  #[inline]
  pub fn detector_id(&self) -> Option<flatbuffers::Vector<'a, i32>> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, i32>>>(SpectraDetectorMapping::VT_DETECTOR_ID, None)}
  }
  #[inline]
  pub fn n_spectra(&self) -> i32 {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<i32>(SpectraDetectorMapping::VT_N_SPECTRA, Some(0)).unwrap()}
  }
}

impl flatbuffers::Verifiable for SpectraDetectorMapping<'_> {
  #[inline]
  fn run_verifier(
    v: &mut flatbuffers::Verifier, pos: usize
  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
    use self::flatbuffers::Verifiable;
    v.visit_table(pos)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, i32>>>("spectrum", Self::VT_SPECTRUM, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, i32>>>("detector_id", Self::VT_DETECTOR_ID, false)?
     .visit_field::<i32>("n_spectra", Self::VT_N_SPECTRA, false)?
     .finish();
    Ok(())
  }
}
pub struct SpectraDetectorMappingArgs<'a> {
    pub spectrum: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, i32>>>,
    pub detector_id: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, i32>>>,
    pub n_spectra: i32,
}
impl<'a> Default for SpectraDetectorMappingArgs<'a> {
  #[inline]
  fn default() -> Self {
    SpectraDetectorMappingArgs {
      spectrum: None,
      detector_id: None,
      n_spectra: 0,
    }
  }
}

pub struct SpectraDetectorMappingBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> SpectraDetectorMappingBuilder<'a, 'b> {
  #[inline]
  pub fn add_spectrum(&mut self, spectrum: flatbuffers::WIPOffset<flatbuffers::Vector<'b , i32>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(SpectraDetectorMapping::VT_SPECTRUM, spectrum);
  }
  #[inline]
  pub fn add_detector_id(&mut self, detector_id: flatbuffers::WIPOffset<flatbuffers::Vector<'b , i32>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(SpectraDetectorMapping::VT_DETECTOR_ID, detector_id);
  }
  #[inline]
  pub fn add_n_spectra(&mut self, n_spectra: i32) {
    self.fbb_.push_slot::<i32>(SpectraDetectorMapping::VT_N_SPECTRA, n_spectra, 0);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> SpectraDetectorMappingBuilder<'a, 'b> {
    let start = _fbb.start_table();
    SpectraDetectorMappingBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<SpectraDetectorMapping<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

impl core::fmt::Debug for SpectraDetectorMapping<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut ds = f.debug_struct("SpectraDetectorMapping");
      ds.field("spectrum", &self.spectrum());
      ds.field("detector_id", &self.detector_id());
      ds.field("n_spectra", &self.n_spectra());
      ds.finish()
  }
}
#[inline]
/// Verifies that a buffer of bytes contains a `SpectraDetectorMapping`
/// and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_spectra_detector_mapping_unchecked`.
pub fn root_as_spectra_detector_mapping(buf: &[u8]) -> Result<SpectraDetectorMapping, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::root::<SpectraDetectorMapping>(buf)
}
#[inline]
/// Verifies that a buffer of bytes contains a size prefixed
/// `SpectraDetectorMapping` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `size_prefixed_root_as_spectra_detector_mapping_unchecked`.
pub fn size_prefixed_root_as_spectra_detector_mapping(buf: &[u8]) -> Result<SpectraDetectorMapping, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::size_prefixed_root::<SpectraDetectorMapping>(buf)
}
#[inline]
/// Verifies, with the given options, that a buffer of bytes
/// contains a `SpectraDetectorMapping` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_spectra_detector_mapping_unchecked`.
pub fn root_as_spectra_detector_mapping_with_opts<'b, 'o>(
  opts: &'o flatbuffers::VerifierOptions,
  buf: &'b [u8],
) -> Result<SpectraDetectorMapping<'b>, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::root_with_opts::<SpectraDetectorMapping<'b>>(opts, buf)
}
#[inline]
/// Verifies, with the given verifier options, that a buffer of
/// bytes contains a size prefixed `SpectraDetectorMapping` and returns
/// it. Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_spectra_detector_mapping_unchecked`.
pub fn size_prefixed_root_as_spectra_detector_mapping_with_opts<'b, 'o>(
  opts: &'o flatbuffers::VerifierOptions,
  buf: &'b [u8],
) -> Result<SpectraDetectorMapping<'b>, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::size_prefixed_root_with_opts::<SpectraDetectorMapping<'b>>(opts, buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a SpectraDetectorMapping and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid `SpectraDetectorMapping`.
pub unsafe fn root_as_spectra_detector_mapping_unchecked(buf: &[u8]) -> SpectraDetectorMapping {
  flatbuffers::root_unchecked::<SpectraDetectorMapping>(buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a size prefixed SpectraDetectorMapping and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid size prefixed `SpectraDetectorMapping`.
pub unsafe fn size_prefixed_root_as_spectra_detector_mapping_unchecked(buf: &[u8]) -> SpectraDetectorMapping {
  flatbuffers::size_prefixed_root_unchecked::<SpectraDetectorMapping>(buf)
}
pub const SPECTRA_DETECTOR_MAPPING_IDENTIFIER: &str = "df12";

#[inline]
pub fn spectra_detector_mapping_buffer_has_identifier(buf: &[u8]) -> bool {
  flatbuffers::buffer_has_identifier(buf, SPECTRA_DETECTOR_MAPPING_IDENTIFIER, false)
}

#[inline]
pub fn spectra_detector_mapping_size_prefixed_buffer_has_identifier(buf: &[u8]) -> bool {
  flatbuffers::buffer_has_identifier(buf, SPECTRA_DETECTOR_MAPPING_IDENTIFIER, true)
}

#[inline]
pub fn finish_spectra_detector_mapping_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<SpectraDetectorMapping<'a>>) {
  fbb.finish(root, Some(SPECTRA_DETECTOR_MAPPING_IDENTIFIER));
}

#[inline]
pub fn finish_size_prefixed_spectra_detector_mapping_buffer<'a, 'b>(fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<SpectraDetectorMapping<'a>>) {
  fbb.finish_size_prefixed(root, Some(SPECTRA_DETECTOR_MAPPING_IDENTIFIER));
}
