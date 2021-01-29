# the-key
Simple fast key generation tool for KV stores

# How to use
```rust
#[macro_use]
use the_key::*;


// Define key parts
define_key_part(Users, &[11]);
define_key_part(Profiles, &[22]);
define_key_part(Photos, &[33]);

// Define keys sequences
define_parts_seq(UsersProfiles, [Users, UsersProfiles]);
define_parts_seq(UsersPhotos, [Users, Photos]);

fn main() {
  let user_id = &[81, 81];
  let profiles = UsersProfiles::new();
  let photos = UsersPhotos::with_suffix(user_id);

  // Get user
  profiles.create_key(user_id.as_ref()); // [11, 22, 81, 81]
  // Get user's photos
  photos.create_prefix(); // [11, 33, 81, 81]
  // Get user's one photo
  photos.create_key(&[99]); // [11, 33, 81, 81, 99]
}
```