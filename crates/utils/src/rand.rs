use base64::alphabet;
use rand::RngExt;

/// Generates a random base64 string of the specified length
pub fn random_b64(len: usize) -> String {
    // Generates a random string using the standard `rng`
    random_b64_with(len, rand::rng())
}

/// Generates a random base64 string of the specified length using the provided [`rand::Rng`]
pub fn random_b64_with<R>(len: usize, rng: R) -> String
where
    R: RngExt,
{
    // Return an empty string immediately if a zero length is received
    if len == 0 {
        return String::new();
    }

    // Creates a character buffer of a specified length
    let mut out = String::with_capacity(len);

    // Creates an iterator of random bytes from the provided `rng`
    let b_gen = rng.random_iter::<u8>();
    let symbols = alphabet::URL_SAFE.symbols();

    // Generates random bytes until the required length is reached
    for b in b_gen {
        // Inserts a byte as a base64 character into the string
        out.push(symbols[(b % 64) as usize].as_char());

        // Stops generation if the string has reached the required length
        if out.len() == len {
            break;
        }
    }

    // Returns the character buffer
    out
}
