mod formatting;

use std::marker::PhantomData;
use formatting::format_struct;

pub trait KeyPart {
  fn get_name_pointer(&self) -> *const &'static str;

  fn get_bytes_pointer(&self) -> *const &'static [u8];
}

pub trait KeyPartsSequence {
  fn get_struct() -> Vec<(*const &'static str, *const &'static [u8])>;

  fn fmt_debug(
    &self,
    parts: &[(*const &'static str, *const &'static [u8])],
    suffix: Option<&[u8]>,
    f: &mut std::fmt::Formatter<'_>
  ) -> std::fmt::Result {
    format_struct(
      parts,
      suffix,
      None,
      f,
    )
  }
}

#[derive(Clone)]
pub struct Key<T: KeyPartsSequence> {
  bytes: Vec<u8>,
  key_len: usize,
  suffix_len: usize,
  phantom: PhantomData<T>,
}

impl<T: KeyPartsSequence> Key<T> {
  pub fn new(bytes: Vec<u8>, suffix_len: usize, key_len: usize) -> Self {
    Self {
      bytes,
      key_len,
      suffix_len,
      phantom: PhantomData,
    }
  }

  pub fn get_key(&self) -> Option<&[u8]> {
    let key = &self.bytes[self.bytes.len() - self.suffix_len..];

    if key.len() > 0 {
      return Some(key)
    }

    None
  }

  pub fn get_prefix(&self) -> &[u8] {
    &self.bytes[..self.bytes.len() - self.suffix_len - self.key_len]
  }

  pub fn get_suffix(&self) -> Option<&[u8]> {
    let suffix = &self.bytes[self.bytes.len() - self.suffix_len - self.key_len..self.bytes.len() - self.suffix_len];

    if suffix.len() > 0 {
      return Some(suffix)
    }

    None
  }

  pub fn to_vec(self) -> Vec<u8> {
    self.bytes
  }
}

impl<T: KeyPartsSequence> Into<Vec<u8>> for Key<T> {
  fn into(self) -> Vec<u8> {
    self.to_vec()
  }
}

impl<T: KeyPartsSequence> std::fmt::Debug for Key<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let prefix_bytes = T::get_struct();
    let suffix_bytes = {
      let bytes = &self.bytes[self.bytes.len() - self.suffix_len - self.key_len..self.bytes.len() - self.key_len];
      match bytes.len() {
        0 => None,
        _ => Some(bytes),
      }
    };

    format_struct(
      prefix_bytes.as_slice(),
      suffix_bytes,
      Some((self.bytes.as_slice(), self.bytes.len())),
      f
    )
  }
}

impl<T: KeyPartsSequence> AsRef<[u8]> for Key<T> {
  fn as_ref(&self) -> &[u8] {
    self.bytes.as_slice()
  }
}

#[macro_export]
macro_rules! define_key_part {
  ($name:ident, $bytes:expr) => {
    pub struct $name {
      key_part_name: *const &'static str,
      bytes: *const &'static [u8],
    }

    impl KeyPart for $name {
      fn get_name_pointer(&self)  -> *const &'static str {
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
      parts: Vec<(*const &'static str, *const &'static [u8])>,
      len: usize,
      suffix: Option<Vec<u8>>,
    }

    impl $name {
      pub fn new() -> Self {
        Self::create::<Vec<u8>>(None)
      }

      pub fn with_suffix<T: AsRef<[u8]>>(suffix: T) -> Self {
        Self::create(Some(suffix))
      }

      fn create<T: AsRef<[u8]>>(suffix: Option<T>) -> Self {
        let mut parts = Vec::new();
        let mut len: usize = 0;

        $({
          let key_part = $key_part::new();
          let bytes_pointer = key_part.get_bytes_pointer();

          unsafe {
            len += std::slice::from_raw_parts(bytes_pointer, 1)[0].len();
          };

          parts.push((key_part.get_name_pointer(), bytes_pointer));
        })*

        let suffix = suffix.map(|bytes| {
          let mut vec_suffix: Vec<u8> = Vec::with_capacity(bytes.as_ref().len());
          vec_suffix.extend_from_slice(bytes.as_ref());
          vec_suffix
        });

        Self {
          parts,
          len,
          suffix,
        }
      }

      pub fn create_key<T: AsRef<[u8]>>(&self, key: T) -> Key<Self> {
        let suffix_len = self.suffix.as_ref().map(|v| v.len()).unwrap_or(0);
        let key = key.as_ref();
        let mut result_key: Vec<u8> = Vec::with_capacity(self.len + suffix_len + key.len());

        self.parts.iter().for_each(|key_part| {
          let bytes = unsafe {
            std::slice::from_raw_parts(key_part.1, 1)[0]
          };

          result_key.extend_from_slice(bytes);
        });

        if let Some(suffix) = &self.suffix {
          result_key.extend_from_slice(suffix.as_slice());
        }

        result_key.extend_from_slice(key);

        Key::new(
          result_key,
          self.suffix.as_ref().map(|v| v.len()).unwrap_or(0),
          key.len(),
        )
      }

      pub fn create_prefix(&self) -> Key<Self> {
        self.create_key(&[])
      }
    }

    impl KeyPartsSequence for $name {
      fn get_struct() -> Vec<(*const &'static str, *const &'static [u8])> {
        let mut parts = Vec::new();

        $({
          let key = $key_part::new();
          parts.push((key.get_name_pointer(), key.get_bytes_pointer()));
        })*

        parts
      }
    }

    impl std::fmt::Debug for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_debug(
          self.parts.as_slice(),
          self.suffix.as_ref().map(|v| v.as_slice()),
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
  fn key_seq_with_suffix_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let suffix = vec![50, 60];
    let key_seq = MyPrefixSeq::with_suffix(suffix);

    assert_eq!(
      key_seq.suffix,
      Some(vec![50, 60]),
    );

    assert_eq!(
      key_seq.create_key(&[70, 80]).to_vec(),
      vec![10, 20, 30, 40, 50, 60, 70, 80],
    )
  }

  #[test]
  fn key_get_key_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let suffix = vec![50, 60];
    let key_seq = MyPrefixSeq::with_suffix(suffix);
    let key = key_seq.create_key(&[70, 80]);

    let expected: &[u8] = &[70, 80];
    assert_eq!(
      key.get_key(),
      Some(expected),
    )
  }

  #[test]
  fn key_get_prefix_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let suffix = vec![50, 60];
    let key_seq = MyPrefixSeq::with_suffix(suffix);
    let key = key_seq.create_key(&[70, 80]);

    assert_eq!(
      key.get_prefix(),
      &[10, 20, 30, 40],
    )
  }

  #[test]
  fn key_get_suffix_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let suffix = vec![50, 60];
    let key_seq = MyPrefixSeq::with_suffix(suffix);
    let key = key_seq.create_key(&[70, 80]);

    let expected: &[u8] = &[50, 60];
    assert_eq!(
      key.get_suffix(),
      Some(expected)
    )
  }

  #[test]
  fn key_seq_create_prefix_test() {
    define_key_part!(KeyPart1, &[10, 20]);
    define_key_part!(KeyPart2, &[30, 40]);
    define_parts_seq!(MyPrefixSeq, [KeyPart1, KeyPart2]);

    let key_seq = MyPrefixSeq::with_suffix(&[50, 60]);

    assert_eq!(
      key_seq.suffix,
      Some(vec![50, 60]),
    );

    assert_eq!(
      key_seq.create_prefix().to_vec(),
      vec![10, 20, 30, 40, 50, 60]
    )
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
}
