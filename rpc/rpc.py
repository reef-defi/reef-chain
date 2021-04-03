from substrateinterface import SubstrateInterface

import json
types = json.load(open("../types.json", "r"))

ip = "127.0.0.1"
port = 9933


s = SubstrateInterface(
    url=f"http://{ip}:{port}",
    ss58_format=42,
    type_registry=types,
    # type_registry_preset='substrate-node-template'
)

s.chain, s.version, s.properties
s.token_symbol, s.token_decimals

s.get_chain_head()

## for becoming a validator
# s.rpc_request('author_rotateKeys', None)

## send 100 REEF from Alice -> Bob
from substrateinterface import Keypair
alice = Keypair.create_from_uri('//Alice')
bob = Keypair.create_from_uri('//Bob')

call = s.compose_call(
    call_module='Balances',
    call_function='transfer',
    call_params={
        'dest': bob.ss58_address,
        'value': 100 * 10**18
    }
)
extrinsic = s.create_signed_extrinsic(call=call, keypair=alice)
# dry run
s.get_payment_info(call=call, keypair=alice)



## check tx metadata
from substrateinterface import ExtrinsicReceipt
receipt = ExtrinsicReceipt(
    substrate=s,
    extrinsic_hash="0x1856aa746ed0cd0f5b01d4e2a5b68bf5a08a0c66b92aac1b3b3738f4a2d59ef6",
    block_hash="0x3cdc5e4e145a25d0361c66669a062695d0e2a511cdd6d4868cfda80e66cf185c"
)
receipt.is_success, receipt.weight, receipt.total_fee_amount
