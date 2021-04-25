from substrateinterface import SubstrateInterface

import json
types = json.load(open("../assets/types.json", "r"))

# url = "http://127.0.0.1:9933"
url = "https://rpc-testnet.reefscan.com"
# url = "https://rpc.reefscan.com"

s = SubstrateInterface(
    url=url,
    ss58_format=42,
    type_registry=types,
    type_registry_preset='substrate-node-template'
)

s.chain, s.version, s.properties
s.token_symbol, s.token_decimals

s.get_chain_head()

## for becoming a validator
# s.rpc_request('author_rotateKeys', None)

## totalSupply
s.query('Balances', 'TotalIssuance')

## current metadata
s.get_runtime_metadata()

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
# tx dry run
s.get_payment_info(call=call, keypair=alice)


## check tx metadata
from substrateinterface import ExtrinsicReceipt
receipt = ExtrinsicReceipt(
    substrate=s,
    extrinsic_hash="0x1856aa746ed0cd0f5b01d4e2a5b68bf5a08a0c66b92aac1b3b3738f4a2d59ef6",
    block_hash="0x3cdc5e4e145a25d0361c66669a062695d0e2a511cdd6d4868cfda80e66cf185c"
)
receipt.is_success, receipt.weight, receipt.total_fee_amount

## find validators missing blocks

# get current validators
validators = [s.ss58_encode(x) for x in s.query('Session', 'Validators').value]
validators

# get current head
r = s.get_runtime_block(include_author=True)
r
author = r['block']['author']
parent = r['block']['header']['parentHash']

from tqdm import tqdm
active_validators = []
for _ in tqdm(range(0, 100)):
    r = s.get_runtime_block(block_hash=parent, include_author=True)
    author = r['block']['author']
    parent = r['block']['header']['parentHash']
    active_validators.append(author)


# recently stopped validating or offline
set(active_validators) ^ set(validators)

# offline validators
set(validators) - set(active_validators)

# online validator statistics
import collections
c = collections.Counter(active_validators)
c

