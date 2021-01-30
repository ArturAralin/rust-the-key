# the-key
Simple fast key management tool for key-value stores

# How to use
```rust
use the_key::*;

// Define key parts
define_key_part!(Users, &[11, 11]);
define_key_part!(Profiles, &[22, 22]);
define_key_part!(Photos, &[33, 33]);

// Define keys sequences
define_parts_seq!(UsersProfiles, [Users, Profiles]);
define_parts_seq!(UsersPhotos, [Users, Photos]);

fn main() {
  let user_id = &[81, 81];
  let profiles = UsersProfiles::new();
  let photos = UsersPhotos::with_suffix(user_id);

  let user_profile_key = profiles.create_key(user_id);

  // Debug example
  println!("{:?}", user_profile_key); // Users[11, 11] -> Profiles[22, 22] -> Key=[81, 81]

  // Pretty debug example
  println!("{:#?}", user_profile_key);
  // Users[11, 11]
  // └ Profiles[22, 22]
  //   └ Key=[81, 81]

  // User
  user_profile_key.to_vec(); // [11, 11, 22, 22, 81, 81]
  // User's photos
  photos.create_prefix().to_vec(); // [11, 11, 33, 33, 81, 81]
  // User's one photo
  photos.create_key(&[99, 99]).to_vec(); // [11, 11, 33, 33, 81, 81, 99, 99]
}
```