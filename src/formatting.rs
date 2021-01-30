pub fn format_struct(
  parts: &[(*const &'static str, *const &'static [u8])],
  suffix: Option<&[u8]>,
  key: Option<(&[u8], usize)>,
  f: &mut std::fmt::Formatter<'_>
) -> std::fmt::Result {
  let mut prefix_len: usize = 0;
  let mut parts = parts
    .iter()
    .map(|key_part| {
      let (name, bytes) = unsafe {
        let name = std::slice::from_raw_parts(key_part.0, 1)[0];
        let bytes = std::slice::from_raw_parts(key_part.1, 1)[0];

        prefix_len += bytes.len();

        (name, bytes)
      };

      format!("{}{:?}", name, bytes)
    })
    .collect::<Vec<String>>();

  if let Some(suffix) = suffix {
    prefix_len += suffix.len();

    parts.push(format!("Suffix={:?}", suffix));
  }

  if let Some(key) = key {
    parts.push(format!("Key={:?}", &key.0[prefix_len..]));
  }

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
