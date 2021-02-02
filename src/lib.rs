mod formatting;

use formatting::format_struct;
use std::marker::PhantomData;

pub type KeyPartItem = (*const &'static str, *const &'static [u8]);
pub type KeyExtensionsItem = (&'static str, Vec<u8>);

pub trait KeyPart {
  fn get_name_pointer(&self) -> *const &'static str;
  fn get_bytes_pointer(&self) -> *const &'static [u8];
}

pub trait KeyPartsSequence {
  fn get_struct() -> Vec<KeyPartItem>;
  fn get_extensions(&self) -> Option<&[KeyExtensionsItem]>;
  fn extend<B: AsRef<[u8]>>(self, key_part_name: &'static str, bytes: B) -> Self;

  // fn to_vec()

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

  pub fn get_key(&self) -> &[u8] {
    &self.bytes[self.bytes.len() - self.key_len..]
  }

  pub fn get_prefix(&self) -> &[u8] {
    &self.bytes[..self.bytes.len() - self.key_len]
  }

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

// macro_rules! count_items {
//   ($($name:ident),*) => {
//     {
//       let mut count = 0usize;
//       $(
//         let _ = stringify!($name);
//         count += 1;
//       )*
//       count
//     }
//   }
// }

#[macro_export]
macro_rules! define_key_part {
  ($name:ident, $bytes:expr) => {
    pub struct $name {
      key_part_name: *const &'static str,
      bytes: *const &'static [u8],
    }

    impl KeyPart for $name {
      fn get_name_pointer(&self) -> *const &'static str {
        self.key_part_name
      }

      fn get_bytes_pointer(&self) -> *const &'static [u8] {
        self.bytes
      }
    }

    impl $name {
      pub fn new() -> Self {
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

#[macro_export]
macro_rules! define_parts_seq {
  ($name:ident, [$($key_part:ident),*]) => {
  pub struct $name {
    // mb use array [Self; N] to escape heap allocations
    parts: Vec<KeyPartItem>,
    extensions: Option<Vec<KeyExtensionsItem>>,
    len: usize,
  }

  impl $name {
    pub fn new() -> Self {
      let mut parts: Vec<KeyPartItem> = Vec::new();
      let mut len: usize = 0;

      $({
        let key_part = $key_part::new();
        let bytes_pointer = key_part.get_bytes_pointer();

        unsafe {
          len += std::slice::from_raw_parts(bytes_pointer, 1)[0].len();
        };

        parts.push((key_part.get_name_pointer(), bytes_pointer));
      })*

      Self {
        parts,
        extensions: None,
        len,
      }
    }

    pub fn create_key<T: AsRef<[u8]>>(&self, key: T) -> Key<Self> {
      let key = key.as_ref();
      let mut result_key: Vec<u8> = Vec::with_capacity(self.len + key.len());

      self.parts.iter().for_each(|key_part| {
        let bytes = unsafe {
          std::slice::from_raw_parts(key_part.1, 1)[0]
        };

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


    fn to_vec(&self) -> Vec<u8> {
      self.create_key(&[]).to_vec()
    }
  }

  impl KeyPartsSequence for $name {
    fn get_struct() -> Vec<KeyPartItem> {
      let mut parts = Vec::new();

      $({
        let key = $key_part::new();
        parts.push((key.get_name_pointer(), key.get_bytes_pointer()));
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
  }

  impl std::fmt::Debug for $name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      self.fmt_debug(
        self.parts.as_slice(),
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

  #[test]
  fn key_part_test() {
    define_key_part!(KeyPart1, "my_key_part_1".as_bytes());

    let kp = KeyPart1::new();

    assert_eq!(
      unsafe { std::slice::from_raw_parts(kp.key_part_name, 1)[0] },
      "KeyPart1",
    );

    assert_eq!(
      unsafe { std::slice::from_raw_parts(kp.bytes, 1)[0] },
      "my_key_part_1".as_bytes(),
    );
  }

  #[test]
  fn key_from_seq_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    assert_eq!(
      MyPrefixSeq::new().create_key(&[50, 60]).to_vec(),
      vec![10, 20, 30, 40, 50, 60],
    )
  }

  #[test]
  fn key_get_key_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let key_seq = MyPrefixSeq::new();
    let key = key_seq.create_key(&[70, 80]);

    let expected: &[u8] = &[70, 80];
    assert_eq!(key.get_key(), expected);
  }

  #[test]
  fn key_get_prefix_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let key_seq = MyPrefixSeq::new();
    let key = key_seq.create_key(&[70, 80]);

    assert_eq!(key.get_prefix(), &[10, 20, 30, 40])
  }

  #[test]
  fn key_seq_debug() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

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
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2, KeyPart3]);

    assert_eq!(
      format!("{:#?}", MyPrefixSeq::new()),
      "KeyPart1[10, 20]\n  └ KeyPart2[30, 40]\n    └ KeyPart3[50, 60]",
    );
  }

  #[test]
  fn key_seq_extend() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let key_seq = MyPrefixSeq::new()
      .extend("ExtensionPart1", &[50, 60])
      .extend("ExtensionPart2", &[70, 80]);

    assert_eq!(
      format!("{:#?}", key_seq),
      "KeyPart1[10, 20]\n  └ KeyPart2[30, 40]\n    └ ExtensionPart1[50, 60]\n      └ ExtensionPart2[70, 80]",
    );

    let key = key_seq.create_key(&[90, 100]);

    assert_eq!(
      key.as_ref(),
      &[10, 20, 30, 40, 50, 60, 70, 80, 90, 100],
    );

    assert_eq!(
      format!("{:?}", key),
      "KeyPart1[10, 20] -> KeyPart2[30, 40] -> ExtensionPart1[50, 60] -> ExtensionPart2[70, 80] -> Key=[90, 100]",
    );
  }
}
