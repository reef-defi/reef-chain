from substrateinterface import SubstrateInterface

ip = "127.0.0.1"
port = 9933

s = SubstrateInterface(
    url=f"http://{ip}:{port}",
    ss58_format=42,
    type_registry_preset='substrate-node-template'
)

s.chain, s.version, s.properties
s.token_symbol, s.token_decimals

s.get_chain_head()

# for becoming a validator
# s.rpc_request('author_rotateKeys', None)
