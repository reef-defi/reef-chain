use super::*;

use frame_support::{assert_err, assert_ok};

#[test]
fn currency_id_to_bytes_works() {
	assert_eq!(Into::<[u8; 32]>::into(CurrencyId::Token(TokenSymbol::REEF)), [0u8; 32]);

	let mut bytes = [0u8; 32];
	bytes[29..].copy_from_slice(&[0, 1, 0][..]);
	assert_eq!(Into::<[u8; 32]>::into(CurrencyId::Token(TokenSymbol::RUSD)), bytes);
}

#[test]
fn currency_id_try_from_bytes_works() {
	let mut bytes = [0u8; 32];
	bytes[29..].copy_from_slice(&[0, 1, 0][..]);
	assert_ok!(bytes.try_into(), CurrencyId::Token(TokenSymbol::RUSD));

	let mut bytes = [0u8; 32];
	bytes[29..].copy_from_slice(&[0, 6, 0][..]);
	assert_err!(TryInto::<CurrencyId>::try_into(bytes), ());

	let mut bytes = [0u8; 32];
	bytes[29..].copy_from_slice(&[1, 6, 0][..]);
	assert_err!(TryInto::<CurrencyId>::try_into(bytes), ());

	let mut bytes = [0u8; 32];
	bytes[29..].copy_from_slice(&[1, 0, 6][..]);
	assert_err!(TryInto::<CurrencyId>::try_into(bytes), ());
}

#[test]
fn currency_id_encode_decode_bytes_works() {
	let currency_id = CurrencyId::Token(TokenSymbol::RUSD);
	let bytes: [u8; 32] = currency_id.into();
	assert_ok!(bytes.try_into(), currency_id)
}
