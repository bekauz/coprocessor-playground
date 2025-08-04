deploy:
    cargo-valence --socket https://service.coprocessor.valence.zone \
      deploy circuit \
      --controller ./circuits/circuit_a/controller \
      --circuit valence-coprocessor-app-circuit | jq -r '.controller' > .controller

prove:
    cargo-valence --socket https://service.coprocessor.valence.zone \
      prove -j '{"eth_addr":"0x8d41bb082C6050893d1eC113A104cc4C087F2a2a","neutron_addr": "neutron1m6w8n0hluq7avn40hj0n6jnj8ejhykfrwfnnjh"}' \
      -p /var/share/proof.bin \
      $(cat .controller)

get:
    cargo-valence --socket https://service.coprocessor.valence.zone \
      storage \
      -p /var/share/proof.bin \
      $(cat .controller) | jq -r '.data' | base64 -d | jq '.proof' | jq -r

debug:
    curl -X POST https://service.coprocessor.valence.zone/api/registry/controller/$CONTROLLER/witnesses \
    -H "Content-Type: application/json" \
    -d '{"args": {"erc20": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","eth_addr":"0x8d41bb082C6050893d1eC113A104cc4C087F2a2a","neutron_addr": "neutron1m6w8n0hluq7avn40hj0n6jnj8ejhykfrwfnnjh"}}' | jq '.log[0]' | jq -r
