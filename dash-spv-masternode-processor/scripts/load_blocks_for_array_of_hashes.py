import requests
import json
import argparse
from chain import Chain

parser = argparse.ArgumentParser(description='Script so useful.')
parser.add_argument("--hashes", type=str, default='')
parser.add_argument("--file", type=str, default='result.json')
parser.add_argument("--chain", type=str, default='mainnet')
args = parser.parse_args()

hashes = args.hashes.split(',')

filename = args.file
chain = Chain.from_string(args.chain)

blocks = []
for i in hashes:
    r = requests.get(f'https://{chain.api_base}/insight-api-dash/block/{i}')
    block = r.json()
    block_hash = block["hash"]
    block_height = block["height"]
    merkle_root = block["merkleroot"]
    # print('MerkleBlock {{ hash: UInt256::from_hex("{}").unwrap().reverse(), height: {}, merkleroot: UInt256::from_hex("{}").unwrap() }},'.format(
    #         block_hash, block_height, merkle_root))
    print('MerkleBlock::reversed({}, "{}", "{}"), '.format(block_height, block_hash, merkle_root))
    # print('store.put(new StoredBlock(new Block(params, {}, '
    #       'Sha256Hash.wrap(Sha256Hash.wrap("{}").getReversedBytes()), '
    #       'Sha256Hash.wrap("{}"), {}, {}, {}, new ArrayList<>()), new BigInteger(Hex.decode("{}")), {}));'
    #       .format(block["version"], block["previousblockhash"], merkle_root, block["time"], 0, block["nonce"], block["chainwork"], block_height))
    # print('StoredBlock block_{} = store.get(Sha256Hash.wrap(Sha256Hash.wrap("{}").getReversedBytes()));'.format(block_height, block_hash))
    blocks.append(block)

# print('{}'.format(blocks))
with open(f'scripts/{filename}', 'w', encoding='utf-8') as f:
    json.dump(blocks, f, ensure_ascii=False, indent=4)

# Example
# --hashes = '00000000000000024352dba79ed19c04b512b7354a25031b7fbb587cf97e07c4,000000000000002f4e6865301d473f6189601bf46a7856e10bb688778af571c5,000000000000000e1c7aa8d3a097c38e53cbed7fb5b117a4fe2ab356913a5b5e,000000000000000de7e088deaecc760151a4ae00a42c309d804b4a8d76260eaf,0000000000000012c2af64bcaa563c7c0327f1715e5d7c2e92eeca94c0df1652,000000000000001120614e702a170894d0a5df800951261b1b5aad2dc5a2701f,0000000000000027eeeffe75296f6dec0bfde97f2adb82ddc5e950aae445d91f,0000000000000021bae834e455984331015d8743a6114050fb131ad8737e412b,00000000000000146a43018f407969f3f5638af85641ef3304a469b3d9401d48,000000000000002e8970da01b397dcce91cc235f693338444763ef19812e4852,0000000000000035064653f503aa9379a3315537ecd092c3850b07ea1f5f2856,0000000000000023fe98b8ce34d18ddf968acdaa147690fa13a9d4293c96f96a,00000000000000041c604b3ece8be36793544290017b5dc95b7fb12f91f52d76,000000000000002b7153d6a72ab7a22316e8dbf97c94ac9a5a1771656ff4de77,0000000000000003a0a5aa7e1eb73c0f9deab941514287eb2aa65e0430388178,000000000000003c0da7bc78e8fe24f926abdceca452716bcbc40bd1f4df1f86'
