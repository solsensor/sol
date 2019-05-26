use rand::Rng;
use std::iter;

pub fn rand_str() -> String {
    let mut rng = rand::thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric))
        .take(64)
        .collect()
}
