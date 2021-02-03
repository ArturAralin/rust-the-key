pub fn format_struct(
  parts: &[super::KeyPartItem],
  extensions: Option<&[super::KeyExtensionsItem]>,
  key: Option<(&[u8], usize)>,
  f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
  let mut prefix_len: usize = 0;

  let mut parts = parts
    .iter()
    .map(|(name, bytes)| {
      prefix_len += bytes.len();

      format!("{}{:?}", name, bytes)
    })
    .collect::<Vec<String>>();

  extensions.unwrap_or(&[]).iter().for_each(|(name, bytes)| {
    prefix_len += bytes.len();

    parts.push(format!("{}{:?}", name, bytes));
  });

  if let Some(key) = key {
    parts.push(format!("Key={:?}", &key.0[prefix_len..]));
  }

  if f.alternate() {
    let mut i: usize = 0;

    for part in parts.iter() {
      let new_line_symbol = match i {
        0 => "",
        _ => "\n",
      };
      let angle_symbol = match i {
        0 => "",
        _ => "└ ",
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
