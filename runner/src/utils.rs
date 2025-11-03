use primitives::types::PeerNumber;

pub fn get_partner_peer_numbers<const PARTNER_COUNT: usize>(
    local_peer_number: PeerNumber,
) -> [PeerNumber; PARTNER_COUNT] {
    (0..PARTNER_COUNT + 1)
        .filter(|&i| i as PeerNumber != local_peer_number)
        .map(|i| i as PeerNumber)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
