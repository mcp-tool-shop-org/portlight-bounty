import type { SiteConfig } from '@mcptoolshop/site-theme';

export const config: SiteConfig = {
  title: 'Portlight Bounty',
  description:
    'A trade game. Buy, sail, and take a contract. The Rust simulation is the rules. The Godot chart is what you play.',
  logoBadge: 'PB',
  brandName: 'Portlight Bounty',
  repoUrl: 'https://github.com/mcp-tool-shop-org/portlight-bounty',
  footerText:
    'Code is MIT. Pictures are not. Built by <a href="https://mcp-tool-shop.github.io/" style="color:var(--color-muted);text-decoration:underline">MCP Tool Shop</a>',

  hero: {
    badge: 'v0.1.0',
    headline: 'Portlight Bounty',
    headlineAccent: 'Trade, sail, prosper.',
    description:
      'A maritime trade game. The Rust port keeps the rules honest against the Python game. Godot 4.7 draws the chart.',
    primaryCta: { href: '#usage', label: 'Play' },
    secondaryCta: { href: 'handbook/', label: 'Read the Handbook' },
    previews: [
      {
        label: 'Chart',
        code: 'cargo build -p portlight-godot\ngodot --path godot',
      },
      {
        label: 'First voyage',
        code: 'New game, then The Merchant.\nLeave the name as Ada.',
      },
      {
        label: 'Simulation',
        code: 'cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42',
      },
    ],
  },

  sections: [
    {
      kind: 'features',
      id: 'features',
      title: 'What you get',
      subtitle: 'Version 0.1.0 is the first GitHub release.',
      features: [
        {
          title: 'A chart you can sail',
          desc: 'Ports, lanes, a market, contracts, crew, the yard, the harbour desk, hunt, the journal, and a fight at sea.',
        },
        {
          title: 'Rules with a witness',
          desc: 'The simulation is checked against the Python game at commit 9b02494. A number both sides agree on stays.',
        },
        {
          title: 'Pictures you can look at',
          desc: 'The plates ship in the repository. The MIT grant does not cover them. They are not free to reuse.',
        },
      ],
    },
    {
      kind: 'code-cards',
      id: 'usage',
      title: 'Run it',
      cards: [
        {
          title: 'Play the chart',
          code: 'cargo build -p portlight-godot\ngodot --path godot',
        },
        {
          title: 'Check the simulation',
          code: 'cargo test --locked --workspace --exclude portlight-godot',
        },
      ],
    },
  ],
};
