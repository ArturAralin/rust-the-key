//! This library provides easy api for key management for key-value storages
//!
//! # Example
//! ```
//! use the_key::*;
//! // Define key parts
//! define_key_part!(Users, &[11, 11]);
//! define_key_part!(Profiles, &[22, 22]);
//! define_key_part!(Photos, &[33, 33]);
//!
//! // Define keys sequences
//! define_key_seq!(UsersProfiles, [Users, Profiles]);
//! define_key_seq!(UsersPhotos, [Users, Photos]);
//!
//! fn main() {
//!   let user_id = &[81, 81];
//!   let profiles = UsersProfiles::new();
//!   let photos = UsersPhotos::new().extend("UserId", user_id);
//!
//!   let user_profile_key = profiles.create_key(user_id);
//!
//!   // Debug example
//!   assert_eq!(
//!     format!("{:?}", user_profile_key),
//!     "Users[11, 11] -> Profiles[22, 22] -> Key=[81, 81]",
//!   );
//!
//!   // Pretty debug example
//!   println!("{:#?}", user_profile_key);
//!   // Users[11, 11]
//!   // └ Profiles[22, 22]
//!   //   └ Key=[81, 81]
//!
//!   assert_eq!(
//!     format!("{:?}", photos),
//!     "Users[11, 11] -> Photos[33, 33] -> UserId[81, 81]",
//!   );
//!
//!   // User
//!   assert_eq!(
//!     user_profile_key.to_vec(),
//!     vec![11, 11, 22, 22, 81, 81],
//!   );
//!
//!   // User's one photo
//!   assert_eq!(
//!     photos.create_key(&[99, 99]).to_vec(),
//!     vec![11, 11, 33, 33, 81, 81, 99, 99],
//!   );
//! }
//! ```

#![feature(test)]
extern crate test;

mod formatting;

use formatting::format_struct;
use std::marker::PhantomData;

pub type KeyPartItem = (&'static str, &'static [u8]);
pub type KeyExtensionsItem = (&'static str, Vec<u8>);

pub trait KeyPart {
  fn new() -> Self;

  /// Returns key part name
  fn get_name(&self) -> &'static str;

  /// Returns key part bytes
  fn get_bytes(&self) -> &'static [u8];
}

pub trait KeyPartsSequence: Clone {
  #[doc(hidden)]
  fn get_struct() -> Vec<KeyPartItem>;
  #[doc(hidden)]
  fn get_extensions(&self) -> Option<&[KeyExtensionsItem]>;

  fn new() -> Self;

  /// Extends key sequence with a new part
  ///
  /// # Example
  /// ```
  /// use the_key::*;
  /// define_key_part!(Part1, &[10, 20]);
  /// define_key_part!(Part2, &[30, 40]);
  /// define_key_seq!(MyKeySeq, [Part1, Part2]);
  ///
  /// fn main() {
  ///   let key_seq = MyKeySeq::new().extend("Part3", &[50, 60]);
  ///
  ///   assert_eq!(
  ///     key_seq.to_vec(),
  ///     vec![10, 20, 30, 40, 50, 60]
  ///   )
  /// }
  /// ```
  fn extend<B: AsRef<[u8]>>(self, key_part_name: &'static str, bytes: B) -> Self;

  /// Creates new [`the_key::Key`][Key] object
  ///
  /// # Example
  /// ```
  /// use the_key::*;
  /// define_key_part!(Part1, &[10, 20]);
  /// define_key_part!(Part2, &[30, 40]);
  /// define_key_seq!(MyKeySeq, [Part1, Part2]);
  ///
  /// fn main() {
  ///   let key_seq = MyKeySeq::new();
  ///   let key = key_seq.create_key(&[50, 60]);
  ///
  ///   assert_eq!(
  ///     key.to_vec(),
  ///     vec![10, 20, 30, 40, 50, 60],
  ///   );
  /// }
  /// ```
  fn create_key<T: AsRef<[u8]>>(&self, key: T) -> Key<Self>;

  #[doc(hidden)]
  fn fmt_debug(
    &self,
    parts: &[KeyPartItem],
    extensions: Option<&[KeyExtensionsItem]>,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    format_struct(parts, extensions, None, f)
  }
}

#[derive(Clone)]
pub struct Key<'a, T: KeyPartsSequence> {
  bytes: Vec<u8>,
  key_len: usize,
  extensions: Option<&'a [KeyExtensionsItem]>,
  phantom: PhantomData<T>,
}

impl<'a, T: KeyPartsSequence> Key<'a, T> {
  pub fn new(bytes: Vec<u8>, key_len: usize, extensions: Option<&'a [KeyExtensionsItem]>) -> Self {
    Self {
      bytes,
      key_len,
      extensions,
      phantom: PhantomData,
    }
  }

  /// Returns key bytes
  pub fn get_key(&self) -> &[u8] {
    &self.bytes[self.bytes.len() - self.key_len..]
  }

  /// Returns prefix bytes
  pub fn get_prefix(&self) -> &[u8] {
    &self.bytes[..self.bytes.len() - self.key_len]
  }

  /// Moves out key bytes
  pub fn to_vec(self) -> Vec<u8> {
    self.bytes
  }
}

impl<'a, T: KeyPartsSequence> Into<Vec<u8>> for Key<'a, T> {
  fn into(self) -> Vec<u8> {
    self.to_vec()
  }
}

impl<'a, T: KeyPartsSequence> std::fmt::Debug for Key<'a, T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    format_struct(
      T::get_struct().as_slice(),
      self.extensions,
      Some((self.bytes.as_slice(), self.bytes.len())),
      f,
    )
  }
}

impl<'a, T: KeyPartsSequence> AsRef<[u8]> for Key<'a, T> {
  fn as_ref(&self) -> &[u8] {
    self.bytes.as_slice()
  }
}

#[doc(hidden)]
#[macro_export]
macro_rules! count {
  ($($name:ident),*) => {
    {
      let mut count = 0usize;
      $(
        let _ = stringify!($name);
        count += 1;
      )*
      count
    }
  }
}

/// Defines a key part. Each key part is a uniq struct whose implement trait [`the_key::KeyPart`][KeyPart]
///
/// # Example
/// ```
/// use the_key::*;
///
/// define_key_part!(KeyPartName, "key_part_bytes".as_bytes());
/// ```
#[macro_export]
macro_rules! define_key_part {
  ($name:ident, $bytes:expr) => {
    #[derive(Debug)]
    pub struct $name {
      key_part_name: &'static str,
      bytes: &'static [u8],
    }

    impl KeyPart for $name {
      fn new() -> Self {
        $name::new()
      }

      fn get_name(&self) -> &'static str {
        self.key_part_name
      }

      fn get_bytes(&self) -> &'static [u8] {
        self.bytes
      }
    }

    impl $name {
      pub const fn new() -> Self {
        const KEY_PART: &'static [u8] = $bytes;
        const KEY_PART_NAME: &'static str = &stringify!($name);

        Self {
          key_part_name: &KEY_PART_NAME,
          bytes: &KEY_PART,
        }
      }
    }
  };
}

/// Defines a key sequence. Each key part is a uniq struct whose implement trait [`the_key::KeyPartsSequence`][KeyPartsSequence]
///
/// # Example
/// ```
/// use the_key::*;
///
/// define_key_part!(KeyPart1, "key_part_1".as_bytes());
/// define_key_part!(KeyPart2, "key_part_2".as_bytes());
/// define_key_seq!(KeyPartsSeq, [KeyPart1, KeyPart2]);
/// ```
#[macro_export]
macro_rules! define_key_seq {
  ($name:ident, [$($key_part:ident),*]) => {
    #[derive(Clone)]
    pub struct $name {
      parts: [KeyPartItem; $crate::count!($($key_part),*)],
      extensions: Option<Vec<KeyExtensionsItem>>,
      len: usize,
    }

    impl $name {
      pub fn new() -> Self {
        let mut len = 0;
        let parts: [KeyPartItem; $crate::count!($($key_part),*)] = [
          $({
            let key_part = $key_part::new();
            let bytes = key_part.get_bytes();

            len += bytes.len();

            (key_part.get_name(), bytes)
          },)*
        ];

        Self {
          len,
          parts,
          extensions: None,
        }
      }

      // This just a public api
      #[allow(dead_code)]
      fn to_vec(&self) -> Vec<u8> {
        self.create_key(&[]).to_vec()
      }
    }

    impl KeyPartsSequence for $name {
      fn new() -> Self {
        $name::new()
      }

      fn get_struct() -> Vec<KeyPartItem> {
        let mut parts = Vec::new();

        $({
          let key = $key_part::new();
          parts.push((key.get_name(), key.get_bytes()));
        })*

        parts
      }

      fn get_extensions(&self) -> Option<&[KeyExtensionsItem]> {
        self.extensions.as_ref().map(|v| v.as_slice())
      }

      fn extend<B: AsRef<[u8]>>(mut self, key_part_name: &'static str, bytes: B) -> Self {
        let key_bytes = bytes.as_ref().to_vec();
        self.len += key_bytes.len();

        self.extensions = match self.extensions {
          Some(mut extensions) => {
            extensions.push((key_part_name, key_bytes));

            Some(extensions)
          },
          None => Some(vec![(key_part_name, key_bytes)]),
        };

        self
      }

      fn create_key<T: AsRef<[u8]>>(&self, key: T) -> Key<Self> {
        let key = key.as_ref();
        let mut result_key: Vec<u8> = Vec::with_capacity(self.len + key.len());

        self.parts.iter().for_each(|(_, bytes)| {
          result_key.extend_from_slice(bytes);
        });

        if let Some(extensions) = &self.extensions {
          extensions.iter().for_each(|(_, bytes)| {
            result_key.extend_from_slice(bytes);
          });
        }

        result_key.extend_from_slice(key);

        Key::new(
          result_key,
          key.len(),
          self.extensions.as_ref().map(|v| v.as_slice())
        )
      }
    }

    impl std::fmt::Debug for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_debug(
          &self.parts,
          self.extensions.as_ref().map(|v| v.as_slice()),
          f
        )
      }
    }
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use test::Bencher;

  #[test]
  fn key_part_test() {
    define_key_part!(KeyPart1, "my_key_part_1".as_bytes());

    let kp = KeyPart1::new();

    assert_eq!(
      kp.get_name(),
      "KeyPart1",
    );

    assert_eq!(
      kp.get_bytes(),
      b"my_key_part_1",
    );
  }

  #[test]
  fn key_from_seq_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_key_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    assert_eq!(
      MyPrefixSeq::new().create_key(&[50, 60]).to_vec(),
      vec![10, 20, 30, 40, 50, 60],
    )
  }

  #[test]
  fn key_get_key_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_key_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let key_seq = MyPrefixSeq::new();
    let key = key_seq.create_key(&[70, 80]);

    let expected: &[u8] = &[70, 80];
    assert_eq!(key.get_key(), expected);
  }

  #[test]
  fn key_get_prefix_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_key_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let key_seq = MyPrefixSeq::new();
    let key = key_seq.create_key(&[70, 80]);

    assert_eq!(key.get_prefix(), &[10, 20, 30, 40])
  }

  #[test]
  fn key_seq_debug() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_key_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    assert_eq!(
      format!("{:?}", MyPrefixSeq::new()),
      "KeyPart1[10, 20] -> KeyPart2[30, 40]".to_string(),
    );
  }

  #[test]
  fn key_seq_pretty_debug() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_key_part!(KeyPart3, &[50, 60]);
    define_key_seq!(MyPrefixSeq, [KeyPart1, KeyPart2, KeyPart3]);

    assert_eq!(
      format!("{:#?}", MyPrefixSeq::new()),
      "KeyPart1[10, 20]\n  └ KeyPart2[30, 40]\n    └ KeyPart3[50, 60]",
    );
  }

  #[test]
  fn key_seq_extend() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_key_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let key_seq = MyPrefixSeq::new()
      .extend("ExtensionPart1", &[50, 60])
      .extend("ExtensionPart2", &[70, 80]);

    assert_eq!(
      format!("{:#?}", key_seq),
      "KeyPart1[10, 20]\n  └ KeyPart2[30, 40]\n    └ ExtensionPart1[50, 60]\n      └ ExtensionPart2[70, 80]",
    );

    let key = key_seq.create_key(&[90, 100]);

    assert_eq!(key.as_ref(), &[10, 20, 30, 40, 50, 60, 70, 80, 90, 100],);

    assert_eq!(
      format!("{:?}", key),
      "KeyPart1[10, 20] -> KeyPart2[30, 40] -> ExtensionPart1[50, 60] -> ExtensionPart2[70, 80] -> Key=[90, 100]",
    );
  }

  // Benches

  #[bench]
  fn bench_key_parts_spawning(b: &mut Bencher) {
    define_key_part!(KeyPart1, "key_part_1".as_bytes());
    define_key_part!(KeyPart2, "key_part_2".as_bytes());
    define_key_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    b.iter(|| {
      MyPrefixSeq::new();
    })
  }

  #[bench]
  fn bench_key_parts_extending(b: &mut Bencher) {
    define_key_part!(KeyPart1, "key_part_1".as_bytes());
    define_key_part!(KeyPart2, "key_part_2".as_bytes());
    define_key_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    b.iter(|| {
      MyPrefixSeq::new().extend("KeyPart3", "key_part_3".as_bytes())
    })
  }

  #[bench]
  fn bench_create_key(b: &mut Bencher) {
    define_key_part!(KeyPart1, "key_part_1".as_bytes());
    define_key_part!(KeyPart2, "key_part_2".as_bytes());
    define_key_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let seq = &MyPrefixSeq::new();

    b.iter(|| {
      seq.create_key("some_key".as_bytes());
    })
  }

  #[bench]
  fn bench_create_key_with_extending(b: &mut Bencher) {
    define_key_part!(KeyPart1, "key_part_1".as_bytes());
    define_key_part!(KeyPart2, "key_part_2".as_bytes());
    define_key_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    b.iter(|| {
      let seq = MyPrefixSeq::new().extend("KeyPart3", "key_part_3");
      seq.create_key("some_key".as_bytes());
    })
  }
}
