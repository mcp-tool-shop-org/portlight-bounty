//! Cross-port NPC networks.
//!
//! Python keeps the relationships in `content/cross_port_networks.py` and
//! looks them up from `engine/consequences.py` when a trusted captain hears
//! gossip. [`gossip`] draws `rng.choice` on the relationships that touch the
//! port, which is `Random.choice` / [`crate::pyrand::PyRandom::choice_index`].

use crate::content::{self, CrossPortNetworkDef};
use crate::pyrand::PyRandom;

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
/// The draw is the relationship among [`relationships_for_port`], in catalog
/// order. The remote port's display name comes from the world catalog; Python
/// falls back to the port id when that port is missing.
pub fn gossip(port_id: &str, rng: &mut PyRandom) -> Option<NetworkGossip> {
    let rels = relationships_for_port(port_id);
    if rels.is_empty() {
        return None;
    }
    let rel = rels[rng.choice_index(rels.len())];
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
    let remote_port_name = content::content()
        .port(remote_port)
        .map(|port| port.name.as_str())
        .unwrap_or(remote_port);
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
        let mut rng = crate::pyrand::PyRandom::from_seed(1);
        let heard = gossip("porto_novo", &mut rng).expect("relationship");
        let mut again = crate::pyrand::PyRandom::from_seed(1);
        let second = gossip("porto_novo", &mut again).expect("relationship");
        assert_eq!(heard, second);
        assert!(heard.text.contains(&heard.local_name));
        assert!(heard.text.contains(&heard.remote_name));
        assert!(!heard.player_impact.is_empty());
        assert!(gossip("no_such_port", &mut rng).is_none());
    }
}
