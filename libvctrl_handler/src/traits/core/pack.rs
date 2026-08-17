//! Pack file reader/writer traits.
//!
//! # Architecture
//! Packfiles are Git's highly compressed archive format for storing multiple objects.
//! This module defines the contracts for both writing and reading packfiles, isolating
//! the complex delta-compression and indexing logic from the standard object store.
//!
//! # Design Rationale: Streaming I/O
//! Packfiles can contain thousands of objects and span gigabytes. The reader trait
//! returns a `Box<dyn Read>` rather than a `Vec<u8>`. This is a critical architectural
//! decision: it forces streaming deserialization. It allows the engine to resolve
//! deltas and decompress zlib streams on the fly, maintaining a constant memory
//! footprint regardless of the packfile's total size.

use crate::errors::VctrlError;
use std::io::Read;

/// Trait for writing Git pack files.
///
/// # Why this exists
/// Provides the contract for building a packfile. Packfiles are essential for
/// network transfers and repository garbage collection, as they compress objects
/// using delta encoding to save space. Abstracting this into a trait allows the
/// crate to support different compression levels or custom delta algorithms.
///
/// # How it works
/// The writer maintains internal state, tracking the offsets of each written object
/// to build a final index. As objects are written via `write_object`, the implementor
/// compresses the data and appends it to the underlying stream. The `finish` method
/// is required to flush any remaining buffers, write the packfile trailer, and
/// finalize the corresponding index file.
///
/// # Examples
///
/// Implementing the trait for a mock in-memory writer:
///
/// ```
/// # use libvctrl_handler::traits::core::pack::PackWriter;
/// # use libvctrl_handler::VctrlError;
/// # use std::collections::HashMap;
/// #
/// struct MockPackWriter {
///     objects: HashMap<Vec<u8>, Vec<u8>>,
/// }
///
/// impl PackWriter for MockPackWriter {
///     type ObjectId = Vec<u8>;
///
///     fn write_object(&mut self, id: &Self::ObjectId, data: &[u8]) -> Result<(), VctrlError> {
///         self.objects.insert(id.clone(), data.to_vec());
///         Ok(())
///     }
///
///     fn finish(&mut self) -> Result<(), VctrlError> {
///         // In a real impl, this would write the checksum and flush the stream.
///         Ok(())
///     }
/// }
///
/// let mut writer = MockPackWriter { objects: HashMap::new() };
/// writer.write_object(&vec![1, 2, 3], b"blob data")?;
/// writer.finish()?;
/// assert_eq!(writer.objects.len(), 1);
/// # Ok::<(), VctrlError>(())
/// ```
pub trait PackWriter: Send + Sync {
    /// The object identifier type.
    ///
    /// # Why this exists
    /// Allows the writer backend to define its own representation of an object hash,
    /// ensuring compatibility with the associated `ObjectStore` implementation.
    type ObjectId: Send + Sync;

    /// Writes an object to the pack.
    ///
    /// # How it works
    /// Accepts an identifier and the raw, uncompressed byte slice of the object.
    /// The implementor is responsible for compressing the data (e.g., using zlib),
    /// calculating offsets, and potentially encoding the object as a delta against
    /// a previously written base object. Requires `&mut self` because writing
    /// mutates the packfile's internal offset tracker and compression state.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if an I/O error occurs during writing or if the
    /// compression algorithm fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::pack::PackWriter;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # struct MockPackWriter { objects: HashMap<Vec<u8>, Vec<u8>> }
    /// # impl PackWriter for MockPackWriter {
    /// #     type ObjectId = Vec<u8>;
    /// #     fn write_object(&mut self, id: &Self::ObjectId, data: &[u8]) -> Result<(), VctrlError> {
    /// #         self.objects.insert(id.clone(), data.to_vec()); Ok(())
    /// #     }
    /// #     fn finish(&mut self) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut writer = MockPackWriter { objects: HashMap::new() };
    /// writer.write_object(&vec![0_u8; 20], b"data")?;
    /// assert!(writer.objects.contains_key(&vec![0_u8; 20]));
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn write_object(&mut self, id: &Self::ObjectId, data: &[u8]) -> Result<(), VctrlError>;

    /// Finishes writing the pack file.
    ///
    /// # How it works
    /// This method must be called exactly once after all objects have been written.
    /// It flushes any remaining data in the compression buffers, writes the 20-byte
    /// SHA-1 trailer for the packfile, and finalizes the index. Failing to call this
    /// method will result in a corrupted, unreadable packfile.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying stream cannot be flushed or if the
    /// final checksum calculation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::pack::PackWriter;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # struct MockPackWriter { objects: HashMap<Vec<u8>, Vec<u8>> }
    /// # impl PackWriter for MockPackWriter {
    /// #     type ObjectId = Vec<u8>;
    /// #     fn write_object(&mut self, id: &Self::ObjectId, data: &[u8]) -> Result<(), VctrlError> {
    /// #         self.objects.insert(id.clone(), data.to_vec()); Ok(())
    /// #     }
    /// #     fn finish(&mut self) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut writer = MockPackWriter { objects: HashMap::new() };
    /// assert!(writer.finish().is_ok());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn finish(&mut self) -> Result<(), VctrlError>;
}

/// Trait for reading Git pack files.
///
/// # Why this exists
/// Provides the contract for random access reading of objects within a packfile.
/// By abstracting this, the crate allows backends to use memory-mapped files,
/// direct file I/O, or entirely in-memory representations for testing.
///
/// # Design Rationale: `&self` and Thread Safety
/// The trait requires `&self` for `read_object` (not `&mut self`). This is crucial
/// for concurrency. Packfiles are immutable once written. By taking an immutable
/// reference, multiple threads can safely read different objects from the same
/// packfile concurrently without requiring external locking.
///
/// # Examples
///
/// Implementing the trait for a mock in-memory reader:
///
/// ```
/// # use libvctrl_handler::traits::core::pack::PackReader;
/// # use libvctrl_handler::VctrlError;
/// # use std::collections::HashMap;
/// # use std::io::{Cursor, Read};
/// #
/// struct MockPackReader {
///     objects: HashMap<Vec<u8>, Vec<u8>>,
/// }
///
/// impl PackReader for MockPackReader {
///     type ObjectId = Vec<u8>;
///
///     fn read_object(&self, id: &Self::ObjectId) -> Result<Box<dyn Read + Send + '_>, VctrlError> {
///         let data = self.objects.get(id).cloned().unwrap_or_default();
///         Ok(Box::new(Cursor::new(data)))
///     }
/// }
///
/// let reader = MockPackReader { objects: HashMap::from([(vec![1], b"data".to_vec())]) };
/// let mut r = reader.read_object(&vec![1])?;
/// let mut buf = String::new();
/// r.read_to_string(&mut buf)?;
/// assert_eq!(buf, "data");
/// # Ok::<(), VctrlError>(())
/// ```
pub trait PackReader: Send + Sync {
    /// The object identifier type.
    ///
    /// # Why this exists
    /// Matches the identifier type used by the corresponding `PackWriter` and
    /// `ObjectStore`, ensuring type-safe lookups across the storage layer.
    type ObjectId: Send + Sync;

    /// Reads an object from the pack, returning a reader.
    ///
    /// # How it works
    /// Looks up the object's offset in the packfile index, seeks to that position,
    /// and returns a boxed reader. The returned reader handles zlib decompression
    /// and, if the object is stored as a delta, resolves the delta against its base
    /// object lazily as bytes are read. The lifetime `'_` ties the returned reader
    /// to the lifetime of the `PackReader` instance, ensuring the underlying file
    /// handle or memory mapping remains valid.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the object is not found in the pack, if the
    /// data is corrupted, or if an I/O error occurs while seeking or reading.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::pack::PackReader;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # use std::io::{Cursor, Read};
    /// # struct MockPackReader { objects: HashMap<Vec<u8>, Vec<u8>> }
    /// # impl PackReader for MockPackReader {
    /// #     type ObjectId = Vec<u8>;
    /// #     fn read_object(&self, id: &Self::ObjectId) -> Result<Box<dyn Read + Send + '_>, VctrlError> {
    /// #         let data = self.objects.get(id).cloned().unwrap_or_default();
    /// #         Ok(Box::new(Cursor::new(data)))
    /// #     }
    /// # }
    /// let reader = MockPackReader { objects: HashMap::new() };
    /// let result = reader.read_object(&vec![1, 2, 3]);
    /// // Mock returns empty cursor for missing keys, but real impls return ObjectNotFound.
    /// assert!(result.is_ok());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn read_object(&self, id: &Self::ObjectId) -> Result<Box<dyn Read + Send + '_>, VctrlError>;
}
