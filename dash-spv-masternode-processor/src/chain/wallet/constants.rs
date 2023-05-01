use crate::chain::derivation;
use crate::chain::Wallet;

pub trait Constants {
    fn extended_private_key_location_string(&self) -> String;
    fn extended_public_key_location_string(&self) -> String;
    fn standalone_extended_public_key_location_string(&self) -> String;
}


impl Constants for Wallet {
    fn extended_private_key_location_string(&self) -> String {
        derivation::wallet_based_extended_private_key_location_string_for_unique_id(self.unique_id_string())
    }

    fn extended_public_key_location_string(&self) -> String {
        derivation::wallet_based_extended_public_key_location_string_for_unique_id(self.unique_id_string())
    }

    fn standalone_extended_public_key_location_string(&self) -> String {
        derivation::standalone_extended_public_key_location_string_for_unique_id(self.unique_id_string())
    }
}
