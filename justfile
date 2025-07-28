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
