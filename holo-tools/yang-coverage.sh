#/bin/sh

cargo run --bin yang_coverage --\
  -m ietf-interfaces\
  -m ietf-ip\
  -m ietf-routing\
  -m ietf-segment-routing\
  -m ietf-segment-routing-mpls\
  -m ietf-key-chain\
  -m ietf-bfd\
  -m ietf-bfd-ip-mh\
  -m ietf-bfd-ip-sh\
  -m ietf-mpls-ldp\
  -m ietf-ospf\
  -m ietf-ospf-sr\
  -m ietf-ospfv3-extended-lsa\
  -m ietf-rip
