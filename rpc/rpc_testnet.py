from substrateinterface import SubstrateInterface

s = SubstrateInterface(
    url=f"https://rpc-testnet.reefscan.com",
    ss58_format=42,
    type_registry_preset='substrate-node-template'
)

s.chain, s.version, s.properties
s.token_symbol, s.token_decimals

s.get_chain_head()

