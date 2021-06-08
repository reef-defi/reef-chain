use crate::{AccountId, Balance, Currencies, CurrencyId, TokenSymbol};

use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use sp_runtime::traits::{SaturatedConversion};

pub fn set_balance(currency_id: CurrencyId, who: &AccountId, balance: Balance) {
	let _ = <Currencies as MultiCurrencyExtended<_>>::update_balance(currency_id, &who, balance.saturated_into());
	assert_eq!(
		<Currencies as MultiCurrency<_>>::free_balance(currency_id, who),
		balance
	);
}

pub fn set_reef_balance(who: &AccountId, balance: Balance) {
	set_balance(CurrencyId::Token(TokenSymbol::REEF), who, balance)
}
