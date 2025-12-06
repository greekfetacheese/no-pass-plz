use argon2_rs::{Argon2, RECOMMENDED_HASH_LENGTH};
use hmac::{Hmac, Mac};
use secure_types::{SecureArray, SecureString, SecureVec, Zeroize};
use sha3::{Digest, Sha3_512};

pub type Error = Box<dyn std::error::Error>;

/// Estimated time 17 seconds
pub fn fast() -> Argon2 {
   Argon2 {
      m_cost: 2048_000,
      t_cost: 8,
      p_cost: 1,
      hash_length: RECOMMENDED_HASH_LENGTH,
      ..Default::default()
   }
}

/// Estimated time 35 seconds
pub fn normal() -> Argon2 {
   Argon2 {
      m_cost: 4096_000,
      t_cost: 8,
      p_cost: 1,
      hash_length: RECOMMENDED_HASH_LENGTH,
      ..Default::default()
   }
}

/// Estimated time 71 seconds
pub fn slow() -> Argon2 {
   Argon2 {
      m_cost: 8192_000,
      t_cost: 8,
      p_cost: 1,
      hash_length: RECOMMENDED_HASH_LENGTH,
      ..Default::default()
   }
}

/// Estimated time 137 seconds
pub fn very_slow() -> Argon2 {
   Argon2 {
      m_cost: 8192_000,
      t_cost: 16,
      p_cost: 1,
      hash_length: RECOMMENDED_HASH_LENGTH,
      ..Default::default()
   }
}

#[derive(Clone)]
pub struct PasswordDeriver {
   seed: SecureArray<u8, 64>,
   pub argon2: Argon2,
}

impl PasswordDeriver {
   pub fn new(
      username: SecureString,
      password: SecureString,
      confirm_password: SecureString,
      argon2: Argon2,
   ) -> Result<Self, Error> {
      validate_credentials(&username, &password, &confirm_password)?;

      let mut hasher = Sha3_512::new();

      username.unlock_str(|username| {
         hasher.update(username.as_bytes());
      });

      let mut result = hasher.finalize();
      let username_hash = result.to_vec();
      result.zeroize();

      let hash = password.unlock_str(|passwd| argon2.hash_password(passwd, username_hash))?;

      let sec_vec = SecureVec::from_vec(hash)?;
      let seed = SecureArray::try_from(sec_vec)?;

      Ok(Self { seed, argon2 })
   }

   pub fn derive_at(&self, index: u32) -> SecureString {
      let res = self.seed.unlock(|seed| {
         let mut mac = Hmac::<Sha3_512>::new_from_slice(seed).expect("HMAC");
         mac.update(&index.to_be_bytes());
         let mut result = mac.finalize().into_bytes();

         let mut hash = result.to_vec();
         result.zeroize();

         let string = hex::encode(&hash);
         hash.zeroize();

         SecureString::from(string)
      });

      res
   }

   pub fn erase(&mut self) {
      self.seed.erase();
   }
}

fn validate_credentials(
   username: &SecureString,
   password: &SecureString,
   confirm_password: &SecureString,
) -> Result<(), Error> {
   if username.char_len() == 0 {
      return Err("Username is empty".into());
   }

   if password.char_len() == 0 {
      return Err("Password is empty".into());
   }

   let ok = password.unlock_str(|password| {
      confirm_password.unlock_str(|confirm_password| password == confirm_password)
   });

   if !ok {
      return Err("Passwords do not match".into());
   }

   Ok(())
}

#[cfg(test)]
mod tests {
   use super::*;
   use secure_types::SecureString;

   #[test]
   fn test_derive_at() {
      let expected_0 = "24edd00e13bba1a55bf1ec2c74961e5545426e3c9dee7c012a58a7832a53c8ca321a7a8cbe58127b1b927548a1f5378184951b6c7cf3b3f18405677c66bcda4b";
      let expected_1 = "88b87c0e89710317acf5bd6fac23183d418d80ad44a99d066c73bc315753d166b035705b9de3ff2b33bbbd57b92ccb61d1bf94fc4da12378ac193e4fe56f27f2";
      let expected_2 = "e7df5ee3657bf8b7311f163a20074e4aa65b83c4638daa05aa5cf361d16fde2fc47144d58ed1254cfaa7b8bd7acc6f845ab9c82583073480ed6450a69fe9c4cd";
      let expected_3 = "c72d7aab9ece4e4f6d92aea522078c485449c85bdf3e15f88949dc46d70a16fb62055f57cee70c3fcdd48fff8937cef09dea41697dbd2dad1de6439d279a64cb";
      let expeted_vec = vec![expected_0, expected_1, expected_2, expected_3];

      let m_cost = 16_000;
      let t_cost = 1;
      let p_cost = 1;

      let argon2 = Argon2::new(m_cost, t_cost, p_cost);
      let password_derive = PasswordDeriver::new(
         SecureString::from("username"),
         SecureString::from("password"),
         SecureString::from("password"),
         argon2,
      )
      .unwrap();

      let indexes = vec![0, 1, 2, 3];
      for index in indexes {
         let password = password_derive.derive_at(index);
         let passwd = password.unlock_str(|s| String::from(s));

         assert_eq!(passwd, expeted_vec[index as usize]);

         println!("Passwd at index {} -> {}", index, passwd);
      }
   }
}
