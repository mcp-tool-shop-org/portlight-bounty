//! Cross-port NPC networks.
//!
//! Python keeps the relationships in `content/cross_port_networks.py` and
//! looks them up from `engine/consequences.py` when a trusted captain hears
//! gossip. The draw itself stays with the caller: this module does not touch
//! an RNG. `gossip` builds the same sentence consequences would, given the
//! relationship the caller already chose.

use crate::content::{self, CrossPortNetworkDef};

/// Relationships in one network: `merchant`, `tavern`, `broker`, or `inspector`.
pub fn relationships(network: &str) -> Vec<&'static CrossPortNetworkDef> {
    content::content()
        .cross_port_networks
        .iter()
        .filter(|rel| rel.network == network)
        .collect()
}

pub fn relationships_for_npc(npc_id: &str) -> Vec<&'static CrossPortNetworkDef> {
    content::content()
        .cross_port_networks
        .iter()
        .filter(|rel| rel.npc_a_id == npc_id || rel.npc_b_id == npc_id)
        .collect()
}

pub fn relationships_for_port(port_id: &str) -> Vec<&'static CrossPortNetworkDef> {
    content::content()
        .cross_port_networks
        .iter()
        .filter(|rel| rel.npc_a_port == port_id || rel.npc_b_port == port_id)
        .collect()
}

/// Player-facing effect of every relationship that touches `port_id`.
pub fn player_impacts_at(port_id: &str) -> Vec<&'static str> {
    relationships_for_port(port_id)
        .into_iter()
        .map(|rel| rel.player_impact.as_str())
        .collect()
}

/// The gossip consequence `check_port_consequences` builds after `rng.choice`.
///
/// `index` is the chosen relationship among [`relationships_for_port`].
/// `remote_port_name` is the display name of the other port; Python falls
/// back to the port id when the port is missing.
pub fn gossip(port_id: &str, index: usize, remote_port_name: &str) -> Option<NetworkGossip> {
    let rels = relationships_for_port(port_id);
    let rel = rels.get(index).copied()?;
    let local_here = rel.npc_a_port == port_id;
    let local_name = if local_here {
        rel.npc_a_name.as_str()
    } else {
        rel.npc_b_name.as_str()
    };
    let remote_name = if local_here {
        rel.npc_b_name.as_str()
    } else {
        rel.npc_a_name.as_str()
    };
    let remote_port = if local_here {
        rel.npc_b_port.as_str()
    } else {
        rel.npc_a_port.as_str()
    };
    let text = format!(
        "At the exchange, {local_name} pulls you aside. 'I heard from {remote_name} at {remote_port_name} — they mentioned your name. You're building a reputation, Captain. The people who matter are starting to notice.' A pause. 'That can be good or bad. Depends on what you do next.'"
    );
    Some(NetworkGossip {
        local_name: local_name.to_string(),
        remote_name: remote_name.to_string(),
        remote_port_id: remote_port.to_string(),
        player_impact: rel.player_impact.clone(),
        disposition: rel.disposition.clone(),
        network: rel.network.clone(),
        text,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkGossip {
    pub local_name: String,
    pub remote_name: String,
    pub remote_port_id: String,
    pub player_impact: String,
    pub disposition: String,
    pub network: String,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_networks_cover_the_catalog() {
        assert!(relationships("merchant").len() >= 6);
        assert!(relationships("tavern").len() >= 5);
        assert!(relationships("broker").len() >= 5);
        assert!(relationships("inspector").len() >= 4);
        assert!(content::content().cross_port_networks.len() >= 20);
    }

    #[test]
    fn relationships_stay_on_distinct_known_ports() {
        let ports: Vec<_> = content::content()
            .ports
            .iter()
            .map(|port| port.id.as_str())
            .collect();
        for rel in &content::content().cross_port_networks {
            assert!(
                ports.contains(&rel.npc_a_port.as_str()),
                "{}",
                rel.npc_a_port
            );
            assert!(
                ports.contains(&rel.npc_b_port.as_str()),
                "{}",
                rel.npc_b_port
            );
            assert_ne!(rel.npc_a_port, rel.npc_b_port);
            assert!(matches!(
                rel.disposition.as_str(),
                "allied" | "rival" | "professional" | "personal" | "respected"
            ));
            assert!(matches!(
                rel.network.as_str(),
                "merchant" | "tavern" | "broker" | "inspector"
            ));
            assert!(!rel.player_impact.is_empty());
        }
    }

    #[test]
    fn porto_novo_and_the_iron_rivalry_are_present() {
        assert!(relationships_for_port("porto_novo").len() >= 3);
        assert!(relationships_for_npc("pn_inspector_salva").len() >= 2);
        let iron = relationships("merchant").into_iter().any(|rel| {
            let ids = [rel.npc_a_id.as_str(), rel.npc_b_id.as_str()];
            ids.contains(&"ih_forge_master") && ids.contains(&"ip_foreman_kofi")
        });
        assert!(iron);
    }

    #[test]
    fn gossip_names_the_local_npc() {
        let gossip = gossip("porto_novo", 0, "Sun Harbor").expect("relationship");
        assert!(gossip.text.contains(&gossip.local_name));
        assert!(gossip.text.contains("Sun Harbor"));
        assert!(!gossip.player_impact.is_empty());
    }
}
