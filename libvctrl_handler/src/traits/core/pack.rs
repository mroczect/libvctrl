//! Defines `PackWriter` and `PackReader` traits for packfile operations.
//!
//! # Purpose
//!
//! Packfiles are an efficient binary representation of version control
//! objects. This module defines two traits:
//!
//! - `PackWriter` abstracts the process of packing objects into a binary
//!   stream.
//! - `PackReader` abstracts the process of retrieving objects from a
//!   packfile.
//!
//! These traits allow different packing algorithms and storage backends to
//! be used without coupling the rest of the system to a specific
//! implementation.
//!
//! # Why a separate module
//!
//! Packing and unpacking are distinct concerns from object storage,
//! reference storage, and transport. Keeping them in their own file follows
//! the same pattern as other core traits, enabling independent evolution.
//!
//! # Examples
//!
//! A simple in-memory pack implementation that stores raw bytes:
//!
//! ```
//! use std::collections::HashMap;
//! use libvctrl_handler::{Hash, PackReader, PackWriter, VctrlError};
//!
//! #[derive(Default)]
//! struct MemoryPack {
//!     objects: HashMap<Hash, Vec<u8>>,
//! }
//!
//! impl PackWriter for MemoryPack {
//!     type ObjectId = Hash;
//!
//!     fn write_object(
//!         &mut self,
//!         id: &Self::ObjectId,
//!         data: &[u8],
//!     ) -> Result<(), VctrlError> {
//!         self.objects.insert(*id, data.to_vec());
//!         Ok(())
//!     }
//!
//!     fn finish(&mut self) -> Result<(), VctrlError> {
//!         Ok(())
//!     }
//! }
//!
//! impl PackReader for MemoryPack {
//!     type ObjectId = Hash;
//!
//!     fn read_object(
//!         &self,
//!         id: &Self::ObjectId,
//!     ) -> Result<Vec<u8>, VctrlError> {
//!         self.objects
//!             .get(id)
//!             .cloned()
//!             .ok_or_else(|| VctrlError::ObjectNotFound(*id))
//!     }
//! }
//!
//! let mut pack = MemoryPack::default();
//! let hash = Hash::from_bytes(&[1u8; 64]).unwrap();
//! pack.write_object(&hash, b"object data").unwrap();
//! pack.finish().unwrap();
//!
//! let data = pack.read_object(&hash).unwrap();
//! assert_eq!(data, b"object data");
//! ```

use crate::{Hash, VctrlError};

/// Trait for writing objects to a packfile stream.
///
/// # Purpose
///
/// `PackWriter` abstracts the process of packing objects into an efficient
/// binary representation. Implementations may buffer objects in memory,
/// write directly to disk, or stream over a network.
///
/// # Associated Types
///
/// - `ObjectId`: the type used to identify an object (e.g., [`Hash`]).
///
/// # Examples
///
/// A trivial implementation that does nothing:
///
/// ```
/// use libvctrl_handler::{Hash, PackWriter, VctrlError};
///
/// struct NullPackWriter;
///
/// impl PackWriter for NullPackWriter {
///     type ObjectId = Hash;
///
///     fn write_object(&mut self, _id: &Hash, _data: &[u8]) -> Result<(), VctrlError> {
///         Ok(())
///     }
///
///     fn finish(&mut self) -> Result<(), VctrlError> {
///         Ok(())
///     }
/// }
///
/// let mut writer = NullPackWriter;
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// writer.write_object(&hash, b"data").unwrap();
/// writer.finish().unwrap();
/// ```
///
/// # Errors
///
/// - [`VctrlError::Other`] if the underlying pack backend fails.
pub trait PackWriter {
    /// The type used to identify an object.
    type ObjectId;

    /// Writes a single object to the pack stream.
    ///
    /// # Parameters
    ///
    /// - `id`: the object identifier.
    /// - `data`: the raw object bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the object cannot be written.
    fn write_object(&mut self, id: &Self::ObjectId, data: &[u8]) -> Result<(), VctrlError>;

    /// Finalizes the pack stream.
    ///
    /// After this method returns successfully, the packfile is complete and
    /// can be read back using a [`PackReader`] implementation.
    ///
    /// # Errors
    ///
    /// Returns an error if the pack cannot be finalized.
    fn finish(&mut self) -> Result<(), VctrlError>;
}

/// Trait for reading objects from a packfile.
///
/// # Purpose
///
/// `PackReader` abstracts the process of retrieving objects from a packfile.
/// Implementations may read from memory, disk, or network.
///
/// # Associated Types
///
/// - `ObjectId`: the type used to identify an object (e.g., [`Hash`]).
///
/// # Examples
///
/// A trivial implementation that always reports missing objects:
///
/// ```
/// use libvctrl_handler::{Hash, PackReader, VctrlError};
///
/// struct EmptyPackReader;
///
/// impl PackReader for EmptyPackReader {
///     type ObjectId = Hash;
///
///     fn read_object(&self, id: &Hash) -> Result<Vec<u8>, VctrlError> {
///         Err(VctrlError::ObjectNotFound(*id))
///     }
/// }
///
/// let reader = EmptyPackReader;
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// assert!(reader.read_object(&hash).is_err());
/// ```
///
/// # Errors
///
/// - [`VctrlError::ObjectNotFound`] if the requested object does not exist.
/// - [`VctrlError::CorruptedData`] if the packfile is malformed.
pub trait PackReader {
    /// The type used to identify an object.
    type ObjectId;

    /// Retrieves an object from the packfile.
    ///
    /// # Parameters
    ///
    /// - `id`: the object identifier to look up.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ObjectNotFound`] if the object does not exist,
    /// or an error if the packfile cannot be read or is corrupted.
    fn read_object(&self, id: &Self::ObjectId) -> Result<Vec<u8>, VctrlError>;
}
