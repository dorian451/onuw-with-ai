use std::collections::HashMap;

use once_cell::sync::Lazy;

pub static ROLE_IMAGE_VARIANTS: Lazy<HashMap<&str, usize>> = Lazy::new(|| {
    [
        ("Alien", 2),
        ("Robber", 2),
        ("Seer", 2),
        ("Tanner", 2),
        ("Troublemaker", 2),
        ("Vampire", 2),
        ("Werewolf", 3),
    ]
    .into_iter()
    .collect()
});

