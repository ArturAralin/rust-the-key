#[macro_export]
macro_rules! define_key_part {
  ($name:ident, $bytes:expr) => {
    pub struct $name {
      key_part_name: *const &'static str,
      bytes: *const &'static [u8],
    }

    impl $name {
      fn new() -> Self {
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
    #[derive(Clone)]
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
          let key = $key_part::new();

          unsafe {
            len += std::slice::from_raw_parts(key.bytes, 1)[0].len();
          };

          parts.push((key.key_part_name, key.bytes));
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

      pub fn create_key<T: AsRef<[u8]>>(&self, key: T) -> Vec<u8> {
        let suffix_len = self.suffix.as_ref().map(|v| v.len()).unwrap_or(0);
        let mut result_key: Vec<u8> = Vec::with_capacity(self.len + suffix_len + key.as_ref().len());

        self.parts.iter().for_each(|key_part| {
          let bytes = unsafe {
            std::slice::from_raw_parts(key_part.1, 1)[0]
          };

          result_key.extend_from_slice(bytes);
        });

        if let Some(suffix) = &self.suffix {
          result_key.extend_from_slice(suffix.as_slice());
        }

        result_key.extend_from_slice(key.as_ref());

        result_key
      }

      pub fn create_prefix(&self) -> Vec<u8> {
        self.create_key(&[])
      }
    }

    impl std::fmt::Debug for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts = self.parts
          .iter()
          .map(|key_part| {
            let (name, bytes) = unsafe {
              let name = std::slice::from_raw_parts(key_part.0, 1)[0];
              let bytes = std::slice::from_raw_parts(key_part.1, 1)[0];

              (name, bytes)
            };

            format!("{}{:?}", name, bytes)
          })
          .collect::<Vec<String>>();

        if f.alternate() {
          let mut i: usize = 0;

          for part in parts.iter() {
            let new_line_symbol = match i {
              0 => "",
              _ => "\n"
            };
            let angle_symbol = match i {
              0 => "",
              _ => "└ "
            };
            let padding = " ".repeat(i);

            i += 2;

            let write_result = write!(f, "{}{}{}{}", new_line_symbol, padding, angle_symbol, part);

            if write_result.is_err() {
              return write_result;
            }
          }
        } else {
          return write!(f, "{}", parts.join(" -> "));
        }

        Ok(())
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
      MyPrefixSeq::new().create_key(&[50, 60]),
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
      key_seq.create_key(&[70, 80]),
      vec![10, 20, 30, 40, 50, 60, 70, 80]
    )
  }

  #[test]
  fn key_seq_create_prefix_test() {
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
      key_seq.create_prefix(),
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
